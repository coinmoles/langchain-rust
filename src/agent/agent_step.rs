use crate::{schemas::ToolCall, tools::ToolOutput};

#[derive(Debug, Clone)]
pub struct AgentStep {
    pub tool_call: ToolCall,
    pub output: ToolOutput,
}

impl AgentStep {
    pub fn new(tool_call: ToolCall, output: ToolOutput) -> Self {
        Self { tool_call, output }
    }
}
