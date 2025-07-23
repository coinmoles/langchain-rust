use crate::schemas::ToolCall;

#[derive(Debug, Clone)]
pub struct AgentStep {
    pub tool_call: ToolCall,
    pub result: String,
    pub summary: Option<String>,
}

impl AgentStep {
    pub fn new(tool_call: ToolCall, result: impl Into<String>, summary: Option<String>) -> Self {
        Self {
            tool_call,
            result: result.into(),
            summary,
        }
    }
}
