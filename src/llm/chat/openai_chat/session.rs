use std::borrow::Cow;
use std::collections::HashMap;

use async_openai::config::Config;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionTool,
    ChatCompletionToolType,
};
use async_trait::async_trait;
use uuid::Uuid;

use crate::agent::AgentError;
use crate::llm::chat::chat_history::ChatHistory;
use crate::llm::chat::helper::{
    McpHandleResult, add_force_final_answer_message, handle_mcp_calls, resolve_mcp_tools,
    select_choice, system_message, tool_message,
};
use crate::llm::chat::request::ChatRequest;
use crate::llm::{LLMError, LLMOptions, LlmSession, OpenAIChat, StepBuffer};
use crate::memory::Memory;
use crate::schemas::{
    IntoWithUsage, LLMEvent, LLMOutput, Message, Role, TokenUsage, ToolCall, ToolSpec, WithUsage,
};
use crate::tools::{FunctionTool, ToolError, ToolOutput};

/// A custom [`LlmSession`] implementation for [`OpenAIChat`].
pub struct OpenAiChatSession<'a, C: Config> {
    /// The [`OpenAIChat`] instance that initialized this session.
    llm: &'a OpenAIChat<C>,

    /// The system message.
    system: ChatCompletionRequestSystemMessage,

    /// The messages exchanged in this session.
    messages: Vec<ChatCompletionRequestMessage>,

    /// The specifications of available tools for the session.
    function_specs: Option<Vec<ChatCompletionTool>>,

    /// The mcp functions.
    mcp_functions: HashMap<String, Box<dyn FunctionTool>>,

    /// The [buffer](StepBuffer) for the current step.
    step_buffer: StepBuffer<ChatCompletionMessageToolCall>,

    /// Whether final answer is forced.
    force_final_answer: bool,

    /// The call options to use for the session.
    options: LLMOptions,
}

impl<'a, C: Config + Send + Sync + 'static> OpenAiChatSession<'a, C> {
    /// Constructs a new [`OpenAiChatSession`].
    pub async fn new(
        llm: &'a OpenAIChat<C>,
        prompt: Vec<Message>,
        tool_spec: Option<ToolSpec>,
        options: LLMOptions,
    ) -> Result<Self, AgentError> {
        let (system, messages): (Vec<_>, Vec<_>) =
            prompt.into_iter().partition(|msg| msg.role == Role::System);
        let system = system_message(system.into_iter().map(|m| m.content).collect());
        let messages = messages.into_iter().map(Into::into).collect::<Vec<_>>();

        let (function_specs, mcp_functions) = resolve_mcp_tools(tool_spec).await?;
        let function_specs = (!function_specs.is_empty()).then(|| {
            function_specs
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
        });

        let session = OpenAiChatSession {
            llm,
            system,
            messages,
            function_specs,
            mcp_functions,
            step_buffer: StepBuffer::new(),
            force_final_answer: false,
            options,
        };
        Ok(session)
    }

    /// Builds a chat completion request with the configured call options.
    fn build_request(&self, options: LLMOptions) -> ChatRequest<'_, ChatHistory<'_>> {
        let function_specs = if !self.force_final_answer {
            self.function_specs.as_deref().map(Cow::Borrowed)
        } else {
            None
        };

        let chat_history = if self
            .llm
            .default_options
            .system_is_assistant
            .unwrap_or_default()
        {
            ChatHistory::as_assistant(&self.system, &self.messages)
        } else {
            ChatHistory::new(&self.system, &self.messages)
        };
        self.llm
            .build_request(chat_history, function_specs, options)
    }

    /// Flushes the step buffer and adds the tool call and results to the message history.
    fn flush_step_buffer(&mut self) -> Result<(), ToolError> {
        let Some(buffer) = self.step_buffer.try_take()? else {
            return Ok(());
        };

        let thought = if self.llm.default_options.drop_thought.unwrap_or(true) {
            None
        } else {
            buffer.thought
        };

        let tool_call_message = tool_call_message(thought, buffer.calls);
        let tool_result_messages = buffer
            .results
            .into_iter()
            .map(|(id, (_, output))| tool_message(&id, output).into());

        self.messages.push(tool_call_message);
        self.messages.extend(tool_result_messages);
        Ok(())
    }

    /// Advances the session by one step.
    ///
    /// Calls to mcp tools are handled internally, and only calls to local tools are included in the
    /// output.
    async fn advance_once(&mut self) -> Result<WithUsage<LLMOutput>, AgentError> {
        self.flush_step_buffer()?;

        let request = self.build_request(self.options.clone());
        let response = self.llm.send_request(request).await?;

        let choice = select_choice(response.choices)?;
        if let Some(refusal) = choice.message.refusal {
            return Err(LLMError::Refused(refusal).into());
        }

        let content = choice.message.content;
        let usage = response.usage.map(Into::into);

        // `function_call` is deprecated, but still included here for completeness.
        #[allow(deprecated)]
        let calls = if let Some(calls) = choice.message.tool_calls {
            calls
        } else if let Some(function) = choice.message.function_call {
            vec![ChatCompletionMessageToolCall {
                id: Uuid::new_v4().to_string(),
                r#type: ChatCompletionToolType::Function,
                function,
            }]
        } else if let Some(text) = content {
            return Ok(LLMOutput::new(LLMEvent::Text(text), None).with_usage(usage));
        } else {
            return Err(LLMError::Other(
                "Cannot convert LLM generation result to LLMOutput".into(),
            )
            .into());
        };

        self.step_buffer.init_step(content.clone(), calls.clone());

        let calls: Vec<ToolCall> = calls
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, _>>()
            .map_err(LLMError::ResponseSerdeError)?;

        // Handle mcp function calls here.
        let McpHandleResult {
            results,
            remaining_calls,
        } = handle_mcp_calls(calls, &self.mcp_functions).await;
        for (id, name, result) in results {
            self.add_tool_result(&id, &name, result);
        }

        // Only include the remaining calls (calls to a local function) in the output.
        Ok(LLMOutput::new(LLMEvent::ToolCall(remaining_calls), content).with_usage(usage))
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LlmSession for OpenAiChatSession<'_, C> {
    async fn load_memory(&mut self, memory: &dyn Memory) -> Result<(), AgentError> {
        let memory_messages = memory
            .messages()
            .into_iter()
            .map(ChatCompletionRequestMessage::from);
        self.messages.splice(0..0, memory_messages);
        Ok(())
    }

    async fn advance(&mut self) -> Result<WithUsage<LLMOutput>, AgentError> {
        let mut usage = None;

        let content = loop {
            // `advance_once` only returns tool calls to non-mcp functions.
            let output = self.advance_once().await?;
            usage = TokenUsage::merge_options([&usage, &output.usage]);

            // If all tool calls are calls to tools from mcp tools, advance without returning.
            if matches!(&output.content.event, LLMEvent::ToolCall(calls) if calls.is_empty()) {
                continue;
            }
            break output.content;
        };

        Ok(content.with_usage(usage))
    }

    fn add_tool_result(
        &mut self,
        id: &str,
        tool_name: &str,
        result: Result<ToolOutput, ToolError>,
    ) {
        self.step_buffer.push_result(id, tool_name, result);
    }

    fn force_final_answer(&mut self) {
        add_force_final_answer_message(&mut self.messages);
    }
}

/// Constructs a tool call message (as an assistant message) from a vector of [`ToolCall`]s by
/// formatting them into an appropriate format.
///
/// # Returns
/// - A [`ChatCompletionRequestMessage`].
fn tool_call_message(
    thought: Option<String>,
    tool_calls: Vec<ChatCompletionMessageToolCall>,
) -> ChatCompletionRequestMessage {
    let mut builder = ChatCompletionRequestAssistantMessageArgs::default();

    if let Some(thought) = thought {
        builder.content(thought);
    }
    builder.tool_calls(tool_calls);

    builder.build().expect("All required fields are set").into()
}
