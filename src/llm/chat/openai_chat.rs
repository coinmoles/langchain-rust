use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::CreateChatCompletionStreamResponse;
use async_trait::async_trait;

use super::OpenAIChatBuilder;
use super::helper::select_choice;
use super::request::ChatRequest;
use crate::llm::chat::helper::{generate, map_stream};
use crate::llm::options::CallOptions;
use crate::llm::{LLM, LLMError, LLMOutput, LLMStream, LlmCapabilities, OpenAIModel};
use crate::schemas::{IntoWithUsage, Message, Prompt, Role, ToolSpec, WithUsage};

#[derive(Clone)]
pub struct OpenAIChat<C: Config> {
    client: OpenAIClient<C>,
    model: String,
    call_options: CallOptions,
}

impl<C: Config> OpenAIChat<C> {
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

    fn process_prompt(&self, prompt: Prompt) -> Vec<Message> {
        prompt
            .to_messages()
            .into_iter()
            .map(|mut message| {
                if self.call_options.system_is_assistant && message.role == Role::System {
                    message.role = Role::Ai;
                }
                if self.call_options.drop_thought
                    && message.tool_calls.as_deref().is_some_and(|t| !t.is_empty())
                {
                    message.content = "".into();
                }
                message
            })
            .collect()
    }
}

impl<C: Config + Default> OpenAIChat<C> {
    pub fn builder() -> OpenAIChatBuilder<C> {
        OpenAIChatBuilder::default()
    }
}

impl Default for OpenAIChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            CallOptions::default(),
        )
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for OpenAIChat<C> {
    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities { native_mcp: false }
    }

    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        if tools.as_ref().is_some_and(|t| !t.mcps.is_empty()) {
            return Err(LLMError::Unsupported(
                "OpenAIChat does not support mcp tools natively".into(),
            ));
        }
        let tools = tools.map(|t| t.functions.to_vec());

        let messages = self.process_prompt(prompt);
        let options = self.call_options.clone();
        let stream = self.call_options.stream.unwrap_or(false);
        let request = ChatRequest::new(&self.model, messages, tools)?.with_options(options);
        let response = generate(&self.client, request, stream).await?;

        let choice: async_openai::types::ChatChoice = select_choice(response.choices)
            .ok_or(LLMError::ContentNotFound("No choices".into()))?;

        let output: LLMOutput = choice.message.try_into()?;
        let usage = response.usage.map(Into::into);

        Ok(output.with_usage(usage))
    }

    async fn stream(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<LLMStream, LLMError> {
        if tools.as_ref().is_some_and(|t| !t.mcps.is_empty()) {
            return Err(LLMError::Unsupported(
                "GenericChat does not support mcp tools natively".into(),
            ));
        }
        let tools = tools.map(|t| t.functions.to_vec());

        let messages = self.process_prompt(prompt);
        let options = self.call_options.clone();
        let request = ChatRequest::new(&self.model, messages, tools)?.with_options(options);

        let original_stream = self
            .client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
            .await?;
        let new_stream = map_stream(original_stream);
        Ok(new_stream)
    }

    fn with_options(&mut self, call_options: CallOptions) {
        self.call_options.merge_options(call_options)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use base64::prelude::*;
    use tokio::sync::Mutex;
    use tokio::test;

    use super::*;
    use crate::llm::options::StreamOption;
    use crate::schemas::{ImageContent, Prompt};

    #[test]
    #[ignore]
    async fn test_invoke() {
        let message_complete = Arc::new(Mutex::new(String::new()));
        let call_options = CallOptions::new().with_stream(StreamOption::default());
        // Setup the OpenAI client with the necessary options
        let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_call_options(call_options)
            .build();

        // Define a set of messages to send to the generate function

        // Call the generate function
        match llm.invoke("hola").await {
            Ok(result) => {
                // Print the response from the generate function
                println!("Generate Result: {result:?}");
                println!("Message Complete: {:?}", message_complete.lock().await);
            }
            Err(e) => {
                // Handle any errors
                eprintln!("Error calling generate: {e:?}");
            }
        }
    }

    #[test]
    #[ignore]
    async fn test_generate() {
        // Define the streaming function as an async block without capturing external references
        // directly
        let call_options = CallOptions::new().with_stream(StreamOption::default());
        // Setup the OpenAI client with the necessary options
        let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_call_options(call_options)
            .build();

        // Define a set of messages to send to the generate function
        let prompt = Prompt::single("Hello, how are you?");

        // Call the generate function
        match llm.generate(prompt, None).await {
            Ok(result) => {
                // Print the response from the generate function
                println!("Generate Result: {result:?}");
            }
            Err(e) => {
                // Handle any errors
                eprintln!("Error calling generate: {e:?}");
            }
        }
    }

    #[test]
    #[ignore]
    async fn test_generate_with_image_message() {
        // Setup the OpenAI client with the necessary options
        let open_ai: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt4o.to_string())
            .build();

        // Convert image to base64
        let image = std::fs::read("./src/llm/test_data/example.jpg").unwrap();
        let image_base64 = BASE64_STANDARD.encode(image);

        // Define a set of messages to send to the generate function
        let image_urls = vec![ImageContent::new(format!(
            "data:image/jpeg;base64,{image_base64}"
        ))];
        let prompt = Prompt::new(vec![
            Message::new_human_message("Describe this image"),
            Message::new_human_message("").with_images(image_urls),
        ]);

        // Call the generate function
        let response = open_ai.generate(prompt, None).await.unwrap();
        println!("Response: {response:?}");
    }
}
