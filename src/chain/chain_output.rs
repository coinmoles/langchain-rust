pub use macros::ChainOutput;
use serde::de::DeserializeOwned;

use crate::{
    chain::Ctor,
    output_parser::{parse_partial_json, OutputParseError},
    schemas::ToolCall,
};

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

pub struct PureOutput<O: DeserializeOwned + Send + Sync + 'static>(pub O);

impl<O: DeserializeOwned + Send + Sync + 'static> PureOutput<O> {
    pub fn into_inner(self) -> O {
        self.0
    }

    pub fn as_inner(&self) -> &O {
        &self.0
    }
}

impl<T, O: DeserializeOwned + Send + Sync> ChainOutput<T> for PureOutput<O> {
    fn construct_from_text(output: impl Into<String>) -> Result<Self, OutputParseError> {
        let original: String = output.into();
        let value = parse_partial_json(&original, false)?;
        let deserialized = serde_json::from_value::<O>(value)?;
        Ok(PureOutput(deserialized))
    }
}

pub struct PureOutputCtor<O: DeserializeOwned + Send + Sync>(std::marker::PhantomData<O>);

impl<O: DeserializeOwned + Send + Sync + 'static> Ctor for PureOutputCtor<O> {
    type Target<'a> = PureOutput<O>;
}
