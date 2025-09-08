use async_openai::{
    config::{Config, OpenAIConfig},
    types::CreateChatCompletionStreamResponse,
    Client as OpenAIClient,
};
use async_trait::async_trait;

use crate::{
    llm::{
        chat::helper::{generate, map_stream},
        options::CallOptions,
        LLMError, LLMOutput, LLMStream, LlmCapabilities, OpenAIModel, LLM,
    },
    schemas::{messages::Message, IntoWithUsage, MessageType, Prompt, ToolSpec, WithUsage},
};

use super::{helper::select_choice, request::ChatRequest, OpenAIChatBuilder};

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
        let mut messages = prompt.to_messages();
        for message in messages.iter_mut() {
            if self.call_options.system_is_assistant && message.message_type == MessageType::System
            {
                message.message_type = MessageType::Ai;
            }
        }
        messages
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

    async fn complete(
        &self,
        prompt: Prompt,
        tools: Option<ToolSpec<'_>>,
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

        let result: LLMOutput = choice.message.try_into()?;
        let usage = response.usage.map(Into::into);

        Ok(result.with_usage(usage))
    }

    async fn stream(
        &self,
        prompt: Prompt,
        tools: Option<ToolSpec<'_>>,
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
    use crate::llm::options::StreamOption;
    use crate::schemas::{MessageType, Prompt};

    use super::*;

    use base64::prelude::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tokio::test;

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
        // Define the streaming function as an async block without capturing external references directly
        let call_options = CallOptions::new().with_stream(StreamOption::default());
        // Setup the OpenAI client with the necessary options
        let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_call_options(call_options)
            .build();

        // Define a set of messages to send to the generate function
        let prompt = Prompt::single("Hello, how are you?");

        // Call the generate function
        match llm.complete(prompt, None).await {
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
        let image_urls = vec![format!("data:image/jpeg;base64,{image_base64}")];
        let prompt = Prompt::new(vec![
            Message::new_human_message("Describe this image"),
            Message::new::<&str>(MessageType::Human, "").with_images(image_urls),
        ]);

        // Call the generate function
        let response = open_ai.complete(prompt, None).await.unwrap();
        println!("Response: {response:?}");
    }
}
