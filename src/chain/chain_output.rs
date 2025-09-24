pub use macros::ChainOutput;

use crate::utils::parse::ParseError;
use crate::schemas::ToolCall;

pub trait ChainOutput<I>: Sized + Send + Sync {
    fn from_text(_text: impl Into<String>) -> Result<Self, ParseError> {
        Err(ParseError::InputRequired)
    }

    fn from_text_and_input(
        input: I,
        text: impl Into<String>,
    ) -> Result<Self, (I, ParseError)> {
        match Self::from_text(text) {
            Err(ParseError::InputRequired) => unimplemented!(),
            other => other.map_err(|e| (input, e)),
        }
    }

    fn from_tool_call(
        _thought: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, ParseError> {
        Err(ParseError::UnexpectedToolCall(tool_calls))
    }

    fn from_tool_call_and_input(
        input: I,
        thought: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, (I, ParseError)> {
        Self::from_tool_call(thought, tool_calls).map_err(|e| (input, e))
    }
}

impl<T> ChainOutput<T> for String {
    fn from_text(output: impl Into<String>) -> Result<Self, ParseError> {
        Ok(output.into())
    }
}
