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
        DefaultInstructor, GenericChatBuilder, Instructor, LLMError, LLMOutput, LLMStream,
        LlmCapabilities, OpenAIModel, LLM,
    },
    schemas::{
        messages::Message, FunctionSpec, IntoWithUsage, MessageType, Prompt, ToolSpec, WithUsage,
    },
};

use super::{helper::select_choice, request::ChatRequest};

pub struct GenericChat<C: Config> {
    client: OpenAIClient<C>,
    model: String,
    instructor: Box<dyn Instructor>,
    call_options: CallOptions,
}

impl<C: Config> GenericChat<C> {
    pub fn new<S>(
        client: OpenAIClient<C>,
        model: S,
        instructor: Box<dyn Instructor>,
        call_options: CallOptions,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            client,
            model: model.into(),
            instructor,
            call_options,
        }
    }

    fn process_prompt(&self, prompt: Prompt, tools: Option<&[FunctionSpec]>) -> Vec<Message> {
        prompt
            .to_messages()
            .into_iter()
            .scan(true, |first_system, mut message| {
                // Inject tool instruction into the first system message.
                if *first_system && message.message_type == MessageType::System {
                    if let Some(tools) = tools {
                        let instruction = self.instructor.tool_use_instruction(tools);
                        message.content.push_str(&instruction);
                    }
                    *first_system = false;
                }

                // Change system message to ai message if configured.
                if self.call_options.system_is_assistant
                    && message.message_type == MessageType::System
                {
                    message.message_type = MessageType::Ai;
                }

                // Convert tool call/result messages to normal ai/human messages
                if let Some(tool_calls) = message.tool_calls {
                    message.content = tool_calls
                        .iter()
                        .map(|tc| tc.to_string())
                        .collect::<Vec<_>>()
                        .join("\n");
                    message.tool_calls = None;
                }
                if message.message_type == MessageType::Tool {
                    message.message_type = MessageType::Human;
                }
                Some(message)
            })
            .collect::<Vec<_>>()
    }
}

impl<C: Config + Default> GenericChat<C> {
    pub fn builder() -> GenericChatBuilder<C> {
        GenericChatBuilder::default()
    }
}

impl Default for GenericChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            Box::new(DefaultInstructor),
            CallOptions::default(),
        )
    }
}

impl<C: Config + Clone> Clone for GenericChat<C> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            model: self.model.clone(),
            instructor: self.instructor.clone_box(),
            call_options: self.call_options.clone(),
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
        let options = self.call_options.clone();
        let stream = self.call_options.stream.unwrap_or(false);
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
        let options = self.call_options.clone();
        let request = ChatRequest::new(&self.model, messages, None)?.with_options(options);

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
