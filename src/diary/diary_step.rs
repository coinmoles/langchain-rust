use crate::schemas::ToolCall;

#[derive(Debug, Clone)]
pub struct DiaryStep {
    pub tool_call: ToolCall,
    pub result: String,
}

impl DiaryStep {
    pub fn new(tool_call: ToolCall, result: impl Into<String>) -> Self {
        Self {
            tool_call,
            result: result.into(),
        }
    }
}
