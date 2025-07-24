pub use macros::ChainOutput;

use crate::{output_parser::OutputParseError, schemas::ToolCall};

pub trait ChainOutput<I>: Sized + Send + Sync {
    fn from_text(_text: impl Into<String>) -> Result<Self, OutputParseError> {
        Err(OutputParseError::InputRequired)
    }

    fn from_text_and_input(
        input: I,
        text: impl Into<String>,
    ) -> Result<Self, (I, OutputParseError)> {
        match Self::from_text(text) {
            Err(OutputParseError::InputRequired) => unimplemented!(),
            other => other.map_err(|e| (input, e)),
        }
    }

    fn from_tool_call(tool_calls: Vec<ToolCall>) -> Result<Self, OutputParseError> {
        Err(OutputParseError::UnexpectedToolCall(tool_calls))
    }

    fn from_tool_call_and_input(
        input: I,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, (I, OutputParseError)> {
        Self::from_tool_call(tool_calls).map_err(|e| (input, e))
    }
}

impl<T> ChainOutput<T> for String {
    fn from_text(output: impl Into<String>) -> Result<Self, OutputParseError> {
        Ok(output.into())
    }
}
