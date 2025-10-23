use std::fmt::Write;

use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::{
    ChatCompletionRequestMessage, CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use async_trait::async_trait;
use serde::Serialize;

use crate::agent::AgentError;
use crate::llm::chat::helper::{generate, map_stream, select_choice};
use crate::llm::options::LLMOptions;
use crate::llm::{
    ChatRequest, DefaultInstructor, GenericChatBuilder, GenericChatSession, Instructor, LLM,
    LLMError, LlmSession, OpenAIModel,
};
use crate::schemas::{
    FunctionSpec, IntoWithUsage, LLMOutput, LLMStream, Message, Prompt, Role, ToolSpec, WithUsage,
};

/// The wrapper for generic chat models that support OpenAI API.
///
/// This struct implements tool use by prompting the model to emit tool calls as plain text and
/// manually parsing them. The specific prompt and parsing logic is injected via the [`Instructor`]
/// object.
///
/// While this works with any OpenAI-comptaible model. However, for models that support structured
/// tool call, it is recommended to use [`OpenAIChat`](crate::llm::OpenAIChat) for better
/// reliability.
pub struct GenericChat<C: Config = OpenAIConfig> {
    /// The OpenAI client.
    client: OpenAIClient<C>,
    /// The model id.
    model: String,
    /// The instructor used to create tool use instruction and parse tool calls.
    pub(super) instructor: Box<dyn Instructor>,
    /// The call options for the LLM.
    pub(super) options: LLMOptions,
}

impl<C: Config + Default> GenericChat<C> {
    /// Creates a [`GenericChatBuilder`] to configure a [`GenericChat`].
    ///
    /// This is the same as calling [`GenericChatBuilder::new`].
    ///
    /// # Example
    /// ```rust
    /// use langchain_rust::llm::{GenericChat, GenericChatBuilder};
    ///
    /// let chat: GenericChat = GenericChat::builder().with_model("gpt-4o-mini").build();
    /// ```
    #[must_use]
    pub fn builder() -> GenericChatBuilder<C> {
        GenericChatBuilder::new()
    }
}

impl<C: Config> GenericChat<C> {
    /// Constructs a new [`GenericChat`].
    ///
    /// It is recommended to use [`GenericChat::builder()`] instead of calling the constructor
    /// directly.
    ///
    /// # Example
    /// ```rust
    /// use langchain_rust::llm::{DefaultInstructor, GenericChat, OpenAIModel};
    ///
    /// let chat: GenericChat = GenericChat::new(
    ///     async_openai::Client::new(),
    ///     OpenAIModel::Gpt4oMini,
    ///     Box::new(DefaultInstructor),
    ///     Default::default(),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        client: OpenAIClient<C>,
        model: impl Into<String>,
        instructor: Box<dyn Instructor>,
        options: LLMOptions,
    ) -> Self {
        Self {
            client,
            model: model.into(),
            instructor,
            options,
        }
    }

    /// Process the prompt into messages.
    ///
    /// The processing includes:
    /// - Appending the tool use instruction into the first system message.
    /// - Converting system messages to ai messages if configured.
    /// - Converting tool call/result messages to normal ai/human messages.
    /// - Dropping the thought part of tool call messages if configured.
    fn process_prompt(
        &self,
        prompt: Prompt,
        tools: Option<&[FunctionSpec]>,
    ) -> Vec<ChatCompletionRequestMessage> {
        prompt
            .to_messages()
            .into_iter()
            .scan(true, |first_system, mut message| {
                // Append tool use instruction into the first system message.
                if *first_system && message.role == Role::System {
                    if let Some(tools) = tools {
                        let instruction = self.instructor.tool_use_instruction(tools);
                        message.content.push_str(&instruction);
                    }
                    *first_system = false;
                }

                // Change system message to ai message if configured.
                if self.options.system_is_assistant.unwrap_or(false) && message.role == Role::System
                {
                    message.role = Role::Ai;
                }

                // Convert tool call/result messages to normal ai/human messages
                if let Some(tool_calls) = message.tool_calls {
                    if self.options.drop_thought.unwrap_or(true) && !tool_calls.is_empty() {
                        // Drop the thought part.
                        message.content = String::new()
                    } else {
                        writeln!(message.content).expect("`Write` to `String` never fails");
                    }

                    writeln!(message.content, "```json").expect("`Write` to `String` never fails");
                    for tool_call in &tool_calls {
                        writeln!(message.content, "{tool_call}")
                            .expect("`Write` to `String` never fails");
                    }
                    writeln!(message.content, "```").expect("`Write` to `String` never fails");
                    message.tool_calls = None;
                }
                if message.role == Role::Tool {
                    message.role = Role::Human;
                }

                Some(ChatCompletionRequestMessage::from(message))
            })
            .collect::<Vec<_>>()
    }

    /// Builds a chat completion request with the configured call options.
    pub(super) fn build_request<M: Serialize>(&self, messages: M) -> ChatRequest<'_, M> {
        ChatRequest::new(&self.model, messages, None).with_options(&self.options)
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

    /// Parses the response from the server using the instructor.
    pub(super) fn parse_response(&self, content: String) -> Result<LLMOutput, LLMError> {
        let output = self.instructor.parse_tool_use(content)?;
        Ok(output)
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for GenericChat<C> {
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        if tools.is_some_and(|t| !t.mcps.is_empty()) {
            log::warn!("`GenericChat` does not support mcp tools natively, they will be ignored");
        }

        let function_specs = tools.map(|t| t.functions.as_slice());
        let messages = self.process_prompt(prompt, function_specs);

        let request = self.build_request(messages);
        let response = self.send_request(request).await?;

        let choice = select_choice(response.choices)?;
        if let Some(refusal) = choice.message.refusal {
            return Err(LLMError::Refused(refusal));
        }
        let output = self.parse_response(choice.message.content.unwrap_or_default())?;
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

        let tools = tools.map(|t| t.functions.as_slice());
        let messages = self.process_prompt(prompt, tools);

        let request = self.build_request(messages);
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
        let session = GenericChatSession::new(self, prompt, tool_spec).await?;
        Ok(Box::new(session))
    }

    fn with_options(&mut self, options: LLMOptions) {
        self.options.merge_options(options)
    }
}

impl Default for GenericChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            Box::new(DefaultInstructor),
            LLMOptions::default(),
        )
    }
}

impl<C: Config + Clone> Clone for GenericChat<C> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            model: self.model.clone(),
            instructor: self.instructor.clone_box(),
            options: self.options.clone(),
        }
    }
}
