use std::pin::Pin;

pub use async_openai::config::{AzureConfig, Config, OpenAIConfig};

use async_openai::{
    types::{CreateChatCompletionResponse, CreateChatCompletionStreamResponse},
    Client as OpenAIClient,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

use crate::{
    llm::{options::CallOptions, LLMError, LLMOutput, LLM},
    schemas::{messages::Message, IntoWithUsage, MessageType, StreamData, TokenUsage, WithUsage},
};

use super::{
    helper::{construct_chat_completion_response, select_choice},
    request::OpenAIRequest,
    OpenAIBuilder, OpenAIModel,
};

#[derive(Clone)]
pub struct OpenAI<C: Config> {
    client: OpenAIClient<C>,
    model: String,
    call_options: CallOptions,
}

impl<C: Config> OpenAI<C> {
    pub fn new<S>(client: OpenAIClient<C>, model: S, call_options: CallOptions) -> Self
    where
        S: Into<String>,
    {
        Self {
            client,
            model: model.into(),
            call_options,
        }
    }

    fn process_prompt(&self, prompt: Vec<Message>) -> Vec<Message> {
        if self.call_options.system_is_assistant {
            prompt
                .into_iter()
                .map(|message| match message.message_type {
                    MessageType::System => Message::new_ai_message(message.content.clone()),
                    _ => message.clone(),
                })
                .collect::<Vec<Message>>()
        } else {
            prompt.to_vec()
        }
    }
}

impl<C: Config + Default> OpenAI<C> {
    pub fn builder() -> OpenAIBuilder<C> {
        OpenAIBuilder::default()
    }
}

impl Default for OpenAI<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            CallOptions::default(),
        )
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for OpenAI<C> {
    async fn generate(&self, prompt: Vec<Message>) -> Result<WithUsage<LLMOutput>, LLMError> {
        let messages = self.process_prompt(prompt);

        let client = self.client.clone().with_http_client(
            reqwest::Client::builder()
                .connection_verbose(true)
                .build()?,
        );
        let request = OpenAIRequest::build_request(&self.model, messages, &self.call_options)?;

        let response = match &self.call_options.stream_option {
            Some(stream_option) => {
                let stream = client
                    .chat()
                    .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
                    .await?;

                construct_chat_completion_response(stream, &stream_option.streaming_func).await?
            }
            None => {
                client
                    .chat()
                    .create_byot::<_, CreateChatCompletionResponse>(request)
                    .await?
            }
        };

        let choice: async_openai::types::ChatChoice = select_choice(response.choices)
            .ok_or(LLMError::ContentNotFound("No choices".into()))?;

        let result: LLMOutput = choice.message.try_into()?;
        let usage = response.usage.map(Into::into);

        Ok(result.with_usage(usage))
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
        let messages = self.process_prompt(messages);

        let request = OpenAIRequest::build_request(&self.model, messages, &self.call_options)?;

        let original_stream = self
            .client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
            .await?;

        let new_stream = original_stream.map(|result| match result {
            Ok(completion) => {
                let value_completion = serde_json::to_value(completion).map_err(LLMError::from)?;
                let usage = value_completion.pointer("/usage");
                if usage.is_some() && !usage.unwrap().is_null() {
                    let usage = serde_json::from_value::<TokenUsage>(usage.unwrap().clone())
                        .map_err(LLMError::from)?;
                    return Ok(StreamData::new(value_completion, Some(usage), ""));
                }
                let content = value_completion
                    .pointer("/choices/0/delta/content")
                    .ok_or(LLMError::ContentNotFound(
                        "/choices/0/delta/content".to_string(),
                    ))?
                    .clone();

                Ok(StreamData::new(
                    value_completion,
                    None,
                    content.as_str().unwrap_or(""),
                ))
            }
            Err(e) => Err(LLMError::from(e)),
        });

        Ok(Box::pin(new_stream))
    }

    fn add_call_options(&mut self, call_options: CallOptions) {
        self.call_options.merge_options(call_options)
    }
}

impl<C: Config> OpenAI<C> {}

#[cfg(test)]
mod tests {
    use crate::llm::options::StreamOption;
    use crate::schemas::MessageType;

    use super::*;

    use async_openai::types::{
        ChatChoiceStream, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs,
    };
    use base64::prelude::*;
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::test;

    #[test]
    #[ignore]
    async fn test_invoke() {
        let message_complete = Arc::new(Mutex::new(String::new()));

        // Define the streaming function
        // This function will append the content received from the stream to `message_complete`
        let streaming_func = {
            let message_complete = message_complete.clone();
            move |content: &str| {
                let message_complete = message_complete.clone();
                let content = content.to_owned();
                async move {
                    let mut message_complete_lock = message_complete.lock().await;
                    println!("Content: {:?}", content);
                    message_complete_lock.push_str(&content);
                    Ok(())
                }
            }
        };
        let call_options = CallOptions::new()
            .with_stream(StreamOption::default().with_streaming_func(streaming_func));
        // Setup the OpenAI client with the necessary options
        let open_ai: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_call_options(call_options)
            .build();

        // Define a set of messages to send to the generate function

        // Call the generate function
        match open_ai.invoke("hola").await {
            Ok(result) => {
                // Print the response from the generate function
                println!("Generate Result: {:?}", result);
                println!("Message Complete: {:?}", message_complete.lock().await);
            }
            Err(e) => {
                // Handle any errors
                eprintln!("Error calling generate: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore]
    async fn test_generate_function() {
        let message_complete = Arc::new(Mutex::new(String::new()));

        // Define the streaming function
        // This function will append the content received from the stream to `message_complete`
        let streaming_func = {
            let message_complete = message_complete.clone();
            move |content: &str| {
                let message_complete = message_complete.clone();
                let content = content.to_owned();
                async move {
                    let content = serde_json::from_str::<ChatChoiceStream>(&content).unwrap();
                    if content.finish_reason.is_some() {
                        return Ok(());
                    }
                    let mut message_complete_lock = message_complete.lock().await;
                    println!("Content: {:?}", content);
                    message_complete_lock.push_str(&content.delta.content.unwrap());
                    Ok(())
                }
            }
        };
        // Define the streaming function as an async block without capturing external references directly
        let call_options = CallOptions::new()
            .with_stream(StreamOption::default().with_streaming_func(streaming_func));
        // Setup the OpenAI client with the necessary options
        let open_ai: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_call_options(call_options)
            .build();

        // Define a set of messages to send to the generate function
        let messages = vec![Message::new_human_message("Hello, how are you?")];

        // Call the generate function
        match open_ai.generate(messages).await {
            Ok(result) => {
                // Print the response from the generate function
                println!("Generate Result: {:?}", result);
                println!("Message Complete: {:?}", message_complete.lock().await);
            }
            Err(e) => {
                // Handle any errors
                eprintln!("Error calling generate: {:?}", e);
            }
        }
    }

    #[test]
    #[ignore]
    async fn test_function() {
        let tools = vec![ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(
                FunctionObjectArgs::default()
                    .name("cli")
                    .description("Use the Ubuntu command line to preform any action you wish.")
                    .parameters(json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The raw command you want executed"
                            }
                        },
                        "required": ["command"]
                    }))
                    .build()
                    .expect("Invalid tool"),
            )
            .build()
            .unwrap()];

        let llm: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_call_options(CallOptions::new().with_tools(tools))
            .build();

        let response = llm
            .invoke("Use the command line to create a new rust project. Execute the first command.")
            .await
            .unwrap();
        println!("{}", response)
    }

    #[test]
    #[ignore]
    async fn test_generate_with_image_message() {
        // Setup the OpenAI client with the necessary options
        let open_ai: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt4o.to_string())
            .build();

        // Convert image to base64
        let image = std::fs::read("./src/llm/test_data/example.jpg").unwrap();
        let image_base64 = BASE64_STANDARD.encode(image);

        // Define a set of messages to send to the generate function
        let image_urls = vec![format!("data:image/jpeg;base64,{image_base64}")];
        let messages = vec![
            Message::new_human_message("Describe this image"),
            Message::new::<&str>(MessageType::Human, "").with_images(image_urls),
        ];

        // Call the generate function
        let response = open_ai.generate(messages).await.unwrap();
        println!("Response: {:?}", response);
    }
}
