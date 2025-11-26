use async_trait::async_trait;

use crate::agent::AgentError;
use crate::llm::{LLM, LLMOptions, LlmSession, StepBuffer};
use crate::memory::Memory;
use crate::schemas::{LLMEvent, LLMOutput, Message, Prompt, Role, ToolCall, ToolSpec, WithUsage};
use crate::tools::{ToolError, ToolOutput};
use crate::utils::helper::FORCE_FINAL_ANSWER;

/// A default [`LlmSession`] implementation.
///
/// This implementation uses the [`LLM::generate`] method to generate the next message. This is used
/// by default if [`LLM::begin_session`] is not implemented using a custom session implementation.
///
/// Note that no model-specific fields are preserved here.
pub struct DefaultSession<'a, L: LLM + ?Sized> {
    /// The llm that initialized
    llm: &'a L,

    /// The messages exchanged during the session.
    messages: Vec<Message>,

    /// The specifications of available tools for the session.
    tool_spec: Option<ToolSpec>,

    /// The [buffer](StepBuffer) for the current step.
    step_buffer: StepBuffer<ToolCall>,

    /// The call options to use for the session.
    options: LLMOptions,
}

impl<'a, L: LLM + ?Sized> DefaultSession<'a, L> {
    /// Constructs a new [`DefaultSession`].
    pub fn new(
        llm: &'a L,
        prompt: Vec<Message>,
        tool_spec: Option<ToolSpec>,
        options: LLMOptions,
    ) -> Self {
        Self {
            llm,
            messages: prompt,
            tool_spec,
            step_buffer: StepBuffer::new(),
            options,
        }
    }

    /// Flushes the step buffer and adds the tool call and results to the message history.
    fn flush_step_buffer(&mut self) -> Result<(), ToolError> {
        let Some(buffer) = self.step_buffer.try_take()? else {
            return Ok(());
        };

        let tool_call_message = Message::new_tool_call_message(None, buffer.calls);
        let tool_result_messages = buffer
            .results
            .into_iter()
            .map(|(id, (_, output))| Message::new_tool_message(Some(id), output.to_string()));
        self.messages.push(tool_call_message);
        self.messages.extend(tool_result_messages);
        Ok(())
    }
}

#[async_trait]
impl<L: LLM + ?Sized> LlmSession for DefaultSession<'_, L> {
    async fn load_memory(&mut self, memory: &dyn Memory) -> Result<(), AgentError> {
        let memory_messages = memory.messages();
        self.messages.splice(0..0, memory_messages);
        Ok(())
    }

    async fn advance(&mut self) -> Result<WithUsage<LLMOutput>, AgentError> {
        self.flush_step_buffer()?;

        let response = self
            .llm
            .generate(
                Prompt::new(self.messages.clone()),
                self.tool_spec.as_ref(),
                self.options.clone(),
            )
            .await?;
        if let LLMEvent::ToolCall(calls) = &response.content.event {
            self.step_buffer
                .init_step(response.content.thought.clone(), calls.clone());
        }

        Ok(response)
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
        if let Some(msg) = self.messages.last_mut()
            && msg.role == Role::Ai
        {
            if msg.tool_calls.is_some() {
                msg.tool_calls = None;
                msg.content = "".into();
            }
        } else {
            self.messages.push(Message::new_ai_message(""));
        }
        self.messages
            .push(Message::new_human_message(FORCE_FINAL_ANSWER));
    }
}
