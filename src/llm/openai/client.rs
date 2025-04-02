use std::{fmt, pin::Pin};

pub use async_openai::config::{AzureConfig, Config, OpenAIConfig};

use async_openai::{
    types::{CreateChatCompletionResponse, CreateChatCompletionStreamResponse},
    Client,
};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

use crate::{
    language_models::{llm::LLM, options::CallOptions, LLMError},
    schemas::{
        generate_result::{GenerateResult, TokenUsage},
        messages::Message,
        MessageType, StreamData,
    },
};

use super::{
    helper::{construct_chat_completion_response, select_choice},
    request::OpenAIRequest,
};

#[derive(Clone)]
pub enum OpenAIModel {
    Gpt35,
    Gpt4,
    Gpt4Turbo,
    Gpt4o,
    Gpt4oMini,
}

impl fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenAIModel::Gpt35 => write!(f, "gpt-3.5-turbo"),
            OpenAIModel::Gpt4 => write!(f, "gpt-4"),
            OpenAIModel::Gpt4Turbo => write!(f, "gpt-4-turbo-preview"),
            OpenAIModel::Gpt4o => write!(f, "gpt-4o"),
            OpenAIModel::Gpt4oMini => write!(f, "gpt-4o-mini"),
        }
    }
}

impl From<OpenAIModel> for String {
    fn from(val: OpenAIModel) -> Self {
        val.to_string()
    }
}

#[derive(Clone)]
pub struct OpenAI<C: Config> {
    config: C,
    options: CallOptions,
    model: String,
}

impl<C: Config> OpenAI<C> {
    pub fn new(config: C) -> Self {
        Self {
            config,
            options: CallOptions::default(),
            model: OpenAIModel::Gpt4oMini.to_string(),
        }
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_config(mut self, config: C) -> Self {
        self.config = config;
        self
    }

    pub fn with_options(mut self, options: CallOptions) -> Self {
        self.options = options;
        self
    }

    fn process_prompt(&self, prompt: Vec<Message>) -> Vec<Message> {
        if self.options.system_is_assistant {
            prompt
                .into_iter()
                .map(|message| match message.message_type {
                    MessageType::SystemMessage => {
                        Message::new(MessageType::AIMessage, message.content.clone())
                    }
                    _ => message.clone(),
                })
                .collect::<Vec<Message>>()
        } else {
            prompt.to_vec()
        }
    }
}

impl Default for OpenAI<OpenAIConfig> {
    fn default() -> Self {
        Self::new(OpenAIConfig::default())
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for OpenAI<C> {
    async fn generate(&self, prompt: Vec<Message>) -> Result<GenerateResult, LLMError> {
        let messages = self.process_prompt(prompt);

        let client = Client::with_config(self.config.clone()).with_http_client(
            reqwest::Client::builder()
                .connection_verbose(true)
                .build()?,
        );
        let request = OpenAIRequest::build_request(&self.model, messages, &self.options)?;

        match &self.options.stream_option {
            Some(stream_option) => {
                let stream = client
                    .chat()
                    .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
                    .await?;

                let response =
                    construct_chat_completion_response(stream, &stream_option.streaming_func)
                        .await?;

                let choice: async_openai::types::ChatChoice = select_choice(response.choices)
                    .ok_or(LLMError::ContentNotFound("No choices".into()))?;

                let result =
                    GenerateResult::new(choice.message.try_into()?, response.usage.map(Into::into));

                Ok(result)
            }
            None => {
                let response = client
                    .chat()
                    .create_byot::<_, CreateChatCompletionResponse>(request)
                    .await?;

                let choice: async_openai::types::ChatChoice = select_choice(response.choices)
                    .ok_or(LLMError::ContentNotFound("No choices".into()))?;

                let result =
                    GenerateResult::new(choice.message.try_into()?, response.usage.map(Into::into));

                Ok(result)
            }
        }
    }

    async fn stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
        let messages = self.process_prompt(messages);

        let client = Client::with_config(self.config.clone());
        let request = OpenAIRequest::build_request(&self.model, messages, &self.options)?;

        let original_stream = client
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

    fn add_options(&mut self, options: CallOptions) {
        self.options.merge_options(options)
    }
}

impl<C: Config> OpenAI<C> {}

#[cfg(test)]
mod tests {
    use crate::language_models::options::StreamOption;
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
        let options = CallOptions::new()
            .with_stream(StreamOption::default().with_streaming_func(streaming_func));
        // Setup the OpenAI client with the necessary options
        let open_ai = OpenAI::new(OpenAIConfig::default())
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_options(options);

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
        let options = CallOptions::new()
            .with_stream(StreamOption::default().with_streaming_func(streaming_func));
        // Setup the OpenAI client with the necessary options
        let open_ai = OpenAI::new(OpenAIConfig::default())
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_options(options);

        // Define a set of messages to send to the generate function
        let messages = vec![Message::new(
            MessageType::HumanMessage,
            "Hello, how are you?",
        )];

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

        let llm = OpenAI::default()
            .with_model(OpenAIModel::Gpt35)
            .with_config(OpenAIConfig::new())
            .with_options(CallOptions::new().with_tools(tools));
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
        let open_ai =
            OpenAI::new(OpenAIConfig::default()).with_model(OpenAIModel::Gpt4o.to_string());

        // Convert image to base64
        let image = std::fs::read("./src/llm/test_data/example.jpg").unwrap();
        let image_base64 = BASE64_STANDARD.encode(image);

        // Define a set of messages to send to the generate function
        let image_urls = vec![format!("data:image/jpeg;base64,{image_base64}")];
        let messages = vec![
            Message::new(MessageType::HumanMessage, "Describe this image"),
            Message::new::<&str>(MessageType::HumanMessage, "").with_images(image_urls),
        ];

        // Call the generate function
        let response = open_ai.generate(messages).await.unwrap();
        println!("Response: {:?}", response);
    }
}
