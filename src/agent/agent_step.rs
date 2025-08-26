use crate::schemas::ToolCall;

/// Represents a single step in an agent's execution, including the tool call and its result.
#[derive(Debug, Clone)]
pub struct AgentStep {
    /// The tool call made during this step.
    pub tool_call: ToolCall,
    /// The result of the tool call.
    pub result: String,
    /// An optional summary of the step, providing additional context or information.
    pub summary: Option<String>,
}

impl AgentStep {
    /// Creates a new `AgentStep` with the specified tool call, result, and summary.
    pub fn new(tool_call: ToolCall, result: impl Into<String>, summary: Option<String>) -> Self {
        Self {
            tool_call,
            result: result.into(),
            summary,
        }
    }
}
