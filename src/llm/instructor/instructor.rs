use crate::llm::LLMOutput;
use crate::output_parser::OutputParseError;
use crate::schemas::FunctionSpec;

pub trait Instructor: Send + Sync {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String;

    fn parse_tool_use(&self, output: String) -> Result<LLMOutput, OutputParseError>;

    fn clone_box(&self) -> Box<dyn Instructor>;
}
