use crate::schemas::ToolCall;

#[derive(Debug)]
pub enum AgentEvent {
    Action(Vec<ToolCall>),
    Finish(String),
}
