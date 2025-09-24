use crate::llm::LLMOutput;
use crate::schemas::FunctionSpec;
use crate::utils::parse::ParseError;

/// An instructor that provides tool use instructions and parses tool use outputs.
///
/// For LLMs that do not natively support tool calling, an instructor is used by the
/// [`GenericChat`](crate::llm::GenericChat) to enable the tool call.
pub trait Instructor: Send + Sync {
    /// Generates an instruction for using the provided tools.
    ///
    /// The instruction should define the appropriate schema for tool call as well as descriptions
    /// for the available tools. The instruction is appended at the end of the system message.
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String;

    /// Parses the LLM output into a tool call object.
    fn parse_tool_use(&self, output: String) -> Result<LLMOutput, ParseError>;

    /// Clones the instructor into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Instructor>;
}
