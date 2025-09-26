use std::fmt::Write;

use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::CreateChatCompletionStreamResponse;
use async_trait::async_trait;

use super::helper::select_choice;
use super::request::ChatRequest;
use crate::llm::chat::helper::{generate, map_stream};
use crate::llm::options::LLMOptions;
use crate::llm::{
    DefaultInstructor, GenericChatBuilder, Instructor, LLM, LLMError, LLMOutput, LLMStream,
    LlmCapabilities, OpenAIModel,
};
use crate::schemas::{FunctionSpec, IntoWithUsage, Message, Prompt, Role, ToolSpec, WithUsage};

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
    instructor: Box<dyn Instructor>,
    /// The call options for the LLM.
    options: LLMOptions,
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
    /// Constructs a new [`GenericChat`]
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
    pub fn new<S>(
        client: OpenAIClient<C>,
        model: S,
        instructor: Box<dyn Instructor>,
        options: LLMOptions,
    ) -> Self
    where
        S: Into<String>,
    {
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
    fn process_prompt(&self, prompt: Prompt, tools: Option<&[FunctionSpec]>) -> Vec<Message> {
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
                Some(message)
            })
            .collect::<Vec<_>>()
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

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for GenericChat<C> {
    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities { native_mcp: false }
    }

    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        if tools.is_some_and(|t| !t.mcps.is_empty()) {
            return Err(LLMError::Unsupported(
                "GenericChat does not support mcp tools natively".into(),
            ));
        }
        let tools = tools.map(|t| t.functions.as_slice());

        let messages = self.process_prompt(prompt, tools);
        let options = self.options.clone();
        let stream = self.options.stream.unwrap_or(false);
        let request = ChatRequest::new(&self.model, messages, None)?.with_options(options);
        let response = generate(&self.client, request, stream).await?;

        let choice: async_openai::types::ChatChoice = select_choice(response.choices)
            .ok_or(LLMError::ContentNotFound("No choices".into()))?;

        let result = self
            .instructor
            .parse_tool_use(choice.message.content.unwrap_or_default())?;
        let usage = response.usage.map(Into::into);

        Ok(result.with_usage(usage))
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
        let tools = tools.map(|t| t.functions.as_slice());

        let messages = self.process_prompt(prompt, tools);
        let options = self.options.clone();
        let request = ChatRequest::new(&self.model, messages, None)?.with_options(options);

        let original_stream = self
            .client
            .chat()
            .create_stream_byot::<_, CreateChatCompletionStreamResponse>(request)
            .await?;
        let new_stream = map_stream(original_stream);
        Ok(new_stream)
    }

    fn with_options(&mut self, options: LLMOptions) {
        self.options.merge_options(options)
    }
}
