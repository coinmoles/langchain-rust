use crate::llm::LLMOutput;
use crate::schemas::FunctionSpec;
use crate::utils::parse::ParseError;

pub trait Instructor: Send + Sync {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String;

    fn parse_tool_use(&self, output: String) -> Result<LLMOutput, ParseError>;

    fn clone_box(&self) -> Box<dyn Instructor>;
}
