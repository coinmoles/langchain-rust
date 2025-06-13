use crate::{
    chain::{ChainOutput, Ctor},
    instructor::{DefaultInstructor, Instructor},
    schemas::ToolCall,
};

pub struct AgentOutputCtor;

impl Ctor for AgentOutputCtor {
    type Target<'a> = AgentOutput;
}

#[derive(Debug)]
pub enum AgentOutput {
    Action(Vec<ToolCall>),
    Finish(String),
}

impl<T> ChainOutput<T> for AgentOutput {
    fn construct_from_text(
        text: impl Into<String>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        DefaultInstructor.parse_from_text(text.into())
    }

    fn construct_from_tool_call(
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        Ok(AgentOutput::Action(tool_calls))
    }
}
