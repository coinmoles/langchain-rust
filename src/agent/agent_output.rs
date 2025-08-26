use crate::{
    chain::{ChainOutput, Ctor},
    instructor::{DefaultInstructor, Instructor},
    schemas::ToolCall,
};

/// The LLM output for a single step of agent execution, which can either be a tool call or a final result.
#[derive(Debug, Ctor)]
pub enum AgentOutput {
    Action(Vec<ToolCall>),
    Finish(String),
}

impl<T> ChainOutput<T> for AgentOutput {
    fn from_text(text: impl Into<String>) -> Result<Self, crate::output_parser::OutputParseError> {
        DefaultInstructor.parse_from_text(text.into())
    }

    fn from_tool_call(
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        Ok(AgentOutput::Action(tool_calls))
    }
}
