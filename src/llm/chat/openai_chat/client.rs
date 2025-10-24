use std::borrow::Cow;

use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionTool, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
};
use async_trait::async_trait;
use serde::Serialize;

use super::OpenAIChatBuilder;
use crate::agent::AgentError;
use crate::llm::chat::helper::{generate, map_stream, select_choice};
use crate::llm::chat::request::ChatRequest;
use crate::llm::options::LLMOptions;
use crate::llm::{LLM, LLMError, LlmSession, OpenAIModel, OpenAiChatSession};
use crate::schemas::{IntoWithUsage, LLMOutput, LLMStream, Message, Prompt, ToolSpec, WithUsage};

/// A wrapper for OpenAI chat models.
///
/// This struct implements tool use by relying on the model's structured tool call capabilities.
/// Consequently, it can also be used for non-OpenAI models with native tool call support as well.
#[derive(Clone)]
pub struct OpenAIChat<C: Config = OpenAIConfig> {
    /// The OpenAI client.
    client: OpenAIClient<C>,
    /// The model id.
    model: String,
    /// The call options for the LLM.
    pub(super) options: LLMOptions,
}

impl<C: Config + Default> OpenAIChat<C> {
    /// Creates a [`OpenAIChatBuilder`] to configure an [`OpenAIChat`].
    ///
    /// This is the same as calling [`OpenAIChatBuilder::new`].
    ///
    /// # Example
    /// ```rust
    /// use langchain_rust::llm::{OpenAIChat, OpenAIChatBuilder};
    ///
    /// let chat: OpenAIChat = OpenAIChat::builder().with_model("gpt-4o").build();
    /// ```
    #[must_use]
    pub fn builder() -> OpenAIChatBuilder<C> {
        OpenAIChatBuilder::new()
    }
}

impl<C: Config> OpenAIChat<C> {
    /// Constructs a new [`OpenAIChat`].
    ///
    /// It is recommended to use [`OpenAIChat::builder()`] instead of calling the constructor
    /// directly.
    ///
    /// ```rust
    /// use langchain_rust::llm::{OpenAIChat, OpenAIModel};
    ///
    /// let chat: OpenAIChat = OpenAIChat::new(
    ///     async_openai::Client::default(),
    ///     OpenAIModel::Gpt4o,
    ///     Default::default(),
    /// );
    /// ```
    #[must_use]
    pub fn new<S>(client: OpenAIClient<C>, model: S, options: LLMOptions) -> Self
    where
        S: Into<String>,
    {
        Self {
            client,
            model: model.into(),
            options,
        }
    }

    /// Processes the prompt into messages.
    ///
    /// The processing includes:
    /// - Converting system messages to ai messages if configured.
    /// - Dropping the content of messages with tool calls if configured.
    fn process_prompt(&self, prompt: Prompt) -> Vec<ChatCompletionRequestMessage> {
        prompt
            .to_messages()
            .into_iter()
            .map(|message| {
                ChatCompletionRequestMessage::from(message.process_with_options(&self.options))
            })
            .collect()
    }

    /// Builds a chat completion request with the configured call options.
    pub(super) fn build_request<'a, M: Serialize>(
        &self,
        messages: M,
        function_specs: Option<Cow<'a, [ChatCompletionTool]>>,
    ) -> ChatRequest<'a, M> {
        ChatRequest::new(&self.model, messages, function_specs).with_options(&self.options)
    }

    /// Sends a chat completion request to the server.
    pub(super) async fn send_request<M: Serialize>(
        &self,
        request: ChatRequest<'_, M>,
    ) -> Result<CreateChatCompletionResponse, LLMError> {
        let response = generate(&self.client, request).await?;
        log::trace!("Received response: {response:?}");
        Ok(response)
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for OpenAIChat<C> {
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        if tools.as_ref().is_some_and(|t| !t.mcps.is_empty()) {
            log::warn!("`OpenAIChat` does not support mcp tools natively, they will be ignored");
        }

        let function_specs =
            tools.map(|t| t.functions.clone().into_iter().map(|f| f.into()).collect());
        let messages = self.process_prompt(prompt);

        let request = self.build_request(messages, function_specs);
        let response = self.send_request(request).await?;

        let choice: async_openai::types::ChatChoice = select_choice(response.choices)?;
        if let Some(refusal) = choice.message.refusal {
            return Err(LLMError::Refused(refusal));
        }
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
            log::warn!("`OpenAIChat` does not support mcp tools natively, they will be ignored");
        }

        let function_specs =
            tools.map(|t| t.functions.clone().into_iter().map(|f| f.into()).collect());
        let messages = self.process_prompt(prompt);

        let request = self.build_request(messages, function_specs);

        let original_stream = self
            .client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
            .await?;
        let new_stream = map_stream(original_stream);
        Ok(new_stream)
    }

    async fn begin_session<'a>(
        &'a self,
        prompt: Vec<Message>,
        tool_spec: Option<ToolSpec>,
    ) -> Result<Box<dyn LlmSession + 'a>, AgentError> {
        let session = OpenAiChatSession::new(self, prompt, tool_spec).await?;
        Ok(Box::new(session))
    }

    fn with_options(&mut self, options: LLMOptions) {
        self.options.merge_options(options)
    }
}

impl Default for OpenAIChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            LLMOptions::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use base64::prelude::*;
    use tokio::sync::Mutex;
    use tokio::test;

    use super::*;
    use crate::schemas::{ImageContent, Message, Prompt};

    #[test]
    #[ignore]
    async fn test_invoke() {
        let message_complete = Arc::new(Mutex::new(String::new()));
        let options = LLMOptions::new().with_stream(true);
        // Setup the OpenAI client with the necessary options
        let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_options(options)
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
        let options = LLMOptions::new().with_stream(true);
        // Setup the OpenAI client with the necessary options
        let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
            .with_model(OpenAIModel::Gpt35.to_string()) // You can change the model as needed
            .with_options(options)
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
