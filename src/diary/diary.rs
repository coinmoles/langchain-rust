use crate::schemas::ToolCall;

use super::DiaryStep;

/// Short term memory for the agent
pub trait Diary: Send + Sync {
    fn push_step(&mut self, tool_call: ToolCall, result: String);
    fn get_steps(&self) -> Vec<DiaryStep>;
}
