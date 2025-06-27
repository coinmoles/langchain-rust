pub use macros::ChainOutput;

use crate::{output_parser::OutputParseError, schemas::ToolCall};

pub trait ChainOutput<I>: Sized + Send + Sync {
    fn construct_from_text(_text: impl Into<String>) -> Result<Self, OutputParseError> {
        Err(OutputParseError::InputRequired)
    }

    fn construct_from_text_and_input(
        input: I,
        text: impl Into<String>,
    ) -> Result<Self, (I, OutputParseError)> {
        match Self::construct_from_text(text) {
            Err(OutputParseError::InputRequired) => unimplemented!(),
            other => other.map_err(|e| (input, e)),
        }
    }

    fn construct_from_tool_call(tool_calls: Vec<ToolCall>) -> Result<Self, OutputParseError> {
        Err(OutputParseError::UnexpectedToolCall(tool_calls))
    }

    fn construct_from_tool_call_and_input(
        input: I,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, (I, OutputParseError)> {
        Self::construct_from_tool_call(tool_calls).map_err(|e| (input, e))
    }
}

impl<T> ChainOutput<T> for String {
    fn construct_from_text(output: impl Into<String>) -> Result<Self, OutputParseError> {
        Ok(output.into())
    }
}
