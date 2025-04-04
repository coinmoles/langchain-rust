use crate::schemas::TokenUsage;

use super::AgentEvent;

/// The result of a single step of an agent.
/// Contains agent event (tool call or final answer) and the token usage information.
pub struct AgentResult {
    content: AgentEvent,
    usage: TokenUsage,
}
