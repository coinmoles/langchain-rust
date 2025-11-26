use std::collections::HashMap;

use async_openai::config::Config;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage,
};
use async_trait::async_trait;

use crate::agent::AgentError;
use crate::llm::chat::chat_history::ChatHistory;
use crate::llm::chat::helper::{
    McpHandleResult, add_force_final_answer_message, append_system, handle_mcp_calls,
    resolve_mcp_tools, select_choice, system_message, user_message,
};
use crate::llm::chat::request::ChatRequest;
use crate::llm::{GenericChat, LLMError, LLMOptions, LlmSession, StepBuffer};
use crate::memory::Memory;
use crate::schemas::{
    IntoWithUsage, LLMEvent, LLMOutput, Message, Role, TokenUsage, ToolCall, ToolSpec, WithUsage,
};
use crate::tools::{FunctionTool, ToolError, ToolOutput};
use crate::utils::helper::add_indent;

/// A custom [`LlmSession`] implementation for [`GenericChat`].
pub struct GenericChatSession<'a, C: Config> {
    /// The [`GenericChat`] instance that initialized this session.
    llm: &'a GenericChat<C>,

    /// The system message.
    system: ChatCompletionRequestSystemMessage,

    /// The system message without tool specifications.
    ///
    /// Used once final answer is forced.
    system_without_tools: ChatCompletionRequestSystemMessage,

    /// The messages exchanged during the session. Excludes the system message.
    messages: Vec<ChatCompletionRequestMessage>,

    /// The mcp functions.
    mcp_functions: HashMap<String, Box<dyn FunctionTool>>,

    /// The [buffer](StepBuffer) for the current step.
    step_buffer: StepBuffer<ToolCall>,

    /// Whether final answer is forced.
    force_final_answer: bool,

    /// The call options to use for the session.
    options: LLMOptions,
}

impl<'a, C: Config + Send + Sync + 'static> GenericChatSession<'a, C> {
    /// Constructs a new [`GenericChatSession`].
    pub async fn new(
        llm: &'a GenericChat<C>,
        prompt: Vec<Message>,
        tool_spec: Option<ToolSpec>,
        options: LLMOptions,
    ) -> Result<Self, AgentError> {
        let (system, messages): (Vec<_>, Vec<_>) =
            prompt.into_iter().partition(|msg| msg.role == Role::System);
        let system_without_tools = system_message(system.into_iter().map(|c| c.content).collect());
        let messages = messages.into_iter().map(Into::into).collect::<Vec<_>>();

        let (function_specs, mcp_functions) = resolve_mcp_tools(tool_spec).await?;
        let system = if !function_specs.is_empty() {
            let tool_use_instruction = llm.instructor.tool_use_instruction(&function_specs);
            append_system(system_without_tools.clone(), tool_use_instruction)
        } else {
            system_without_tools.clone()
        };

        let session = GenericChatSession {
            llm,
            system_without_tools,
            system,
            messages,
            mcp_functions,
            step_buffer: StepBuffer::new(),
            force_final_answer: false,
            options,
        };
        Ok(session)
    }

    /// Builds a chat completion request with the configured call options.
    fn build_request(&self, options: LLMOptions) -> ChatRequest<'_, ChatHistory<'_>> {
        let system = if self.force_final_answer {
            &self.system_without_tools
        } else {
            &self.system
        };

        let chat_history = if self
            .llm
            .default_options
            .system_is_assistant
            .unwrap_or_default()
        {
            ChatHistory::as_assistant(system, &self.messages)
        } else {
            ChatHistory::new(system, &self.messages)
        };

        self.llm.build_request(chat_history, options)
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
            .values()
            .map(|(_, output)| user_message(output.to_string()).into());

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

        let output_raw = choice.message.content.clone().unwrap_or_default();
        let output = self.llm.parse_response(output_raw)?;
        let usage = response.usage.map(Into::into);

        let LLMOutput { event, thought } = output;
        let calls = match event {
            LLMEvent::ToolCall(calls) => calls,
            // Returns early if not tool call
            other => return Ok(LLMOutput::new(other, thought).with_usage(usage)),
        };

        self.step_buffer.init_step(thought.clone(), calls.clone());

        // Handle mcp function calls here.
        let McpHandleResult {
            results,
            remaining_calls,
        } = handle_mcp_calls(calls, &self.mcp_functions).await;
        for (id, name, result) in results {
            self.add_tool_result(&id, &name, result);
        }

        // Only include the remaining calls (calls to a local function) in the output.
        Ok(LLMOutput::new(LLMEvent::ToolCall(remaining_calls), thought).with_usage(usage))
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LlmSession for GenericChatSession<'_, C> {
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
    tool_calls: Vec<ToolCall>,
) -> ChatCompletionRequestMessage {
    let tool_calls = if tool_calls.len() == 1 {
        tool_calls
            .first()
            .expect("First element exists")
            .to_string()
    } else {
        let tool_calls = tool_calls
            .iter()
            .map(|call| add_indent(&call.to_string(), 4, true))
            .collect::<Vec<_>>()
            .join("\n");
        format!("[\n{tool_calls}\n]")
    };

    let content = if let Some(thought) = thought {
        format!("{thought}\n```json\n{tool_calls}\n```")
    } else {
        format!("```json\n{tool_calls}\n```")
    };

    ChatCompletionRequestAssistantMessageArgs::default()
        .content(content)
        .build()
        .expect("All required fields are set")
        .into()
}
