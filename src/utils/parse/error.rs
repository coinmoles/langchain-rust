use thiserror::Error;

use crate::schemas::ToolCall;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Deserialization error: {0}\nOriginal: {1}")]
    Deserialize(serde_json::Error, String),

    #[error("Unexpected tool call {0:?}")]
    UnexpectedToolCall(Vec<ToolCall>),

    #[error("Cannot construct input without output")]
    InputRequired,

    #[error("Other error: {0}")]
    Other(String),
}

pub trait ParseResultExt<T, E> {
    fn with_input<I>(self, input: I) -> Result<(I, T), (I, E)>;
}

impl<T, E> ParseResultExt<T, E> for Result<T, E> {
    fn with_input<I>(self, input: I) -> Result<(I, T), (I, E)> {
        match self {
            Ok(value) => Ok((input, value)),
            Err(err) => Err((input, err)),
        }
    }
}
