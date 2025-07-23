use crate::{
    agent::{AgentOutput, AgentOutputCtor},
    chain::{InputCtor, OutputCtor},
    output_parser::{OutputParseError, OutputParser},
    tools::ToolDyn,
};

pub trait Instructor: Send + Sync {
    fn create_suffix(&self, tools: &[&dyn ToolDyn]) -> String;

    fn parse_from_text(&self, output: String) -> Result<AgentOutput, OutputParseError>;
}

pub trait BoxInstructorExt {
    fn into_parser<I: InputCtor>(self) -> Box<dyn OutputParser<I, AgentOutputCtor>>;
}

impl BoxInstructorExt for Box<dyn Instructor> {
    fn into_parser<I: InputCtor>(self) -> Box<dyn OutputParser<I, AgentOutputCtor>> {
        Box::new(InstructParser(self))
    }
}

#[repr(transparent)]
pub struct InstructParser(Box<dyn Instructor>);

impl<I: InputCtor> OutputParser<I, AgentOutputCtor> for InstructParser {
    fn parse_from_text<'a>(
        &self,
        output: String,
    ) -> Result<<AgentOutputCtor as OutputCtor>::Target<'a>, OutputParseError> {
        self.0.parse_from_text(output)
    }
}
