use crate::schemas::ToolCall;

pub struct AgentStep {
    pub tool_call: ToolCall,
    pub result: String,
}

impl AgentStep {
    pub fn new(tool_call: ToolCall, result: impl Into<String>) -> Self {
        Self {
            tool_call,
            result: result.into(),
        }
    }
}
