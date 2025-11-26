use async_trait::async_trait;

use crate::agent::AgentError;
use crate::llm::LLMOptions;
use crate::memory::Memory;
use crate::schemas::{LLMOutput, WithUsage};
use crate::tools::{ToolError, ToolOutput};

/// A prolonged session with an agentic LLM.
///
/// An interaction with an agentic LLM typically has the agent output tool calls to which the user
/// respond with a result of that tool call. Using the schema types defined in this crate for this
/// interaction risks possibly losing out on some information.
///
/// To preserve all information, you can provide a custom implementation of this trait to store them
/// in a model-specific types. Refer to [`GenericChatSession`](crate::llm::GenericChatSession) or
/// [`OpenAIChatSession`](crate::llm::OpenAIChatSession) to get the hang of implementing this
/// trait.
#[async_trait]
pub trait LlmSession: Send + Sync {
    /// Loads messages from long term memory into this session.
    async fn load_memory(&mut self, _memory: &dyn Memory) -> Result<(), AgentError>;

    /// Advances the session until the model requires a tool result from the user.
    async fn advance(&mut self, options: LLMOptions) -> Result<WithUsage<LLMOutput>, AgentError>;

    /// Adds a tool result to the session.
    fn add_tool_result(&mut self, id: &str, tool_name: &str, result: Result<ToolOutput, ToolError>);

    /// Forces the agent to emit a final answer.
    fn force_final_answer(&mut self) {}
}
