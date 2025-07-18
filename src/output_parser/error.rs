use thiserror::Error;

use crate::schemas::ToolCall;

#[derive(Debug, Error)]
pub enum OutputParseError {
    #[error("Deserialization error: {0}\nOriginal: {1}")]
    Deserialize(serde_json::Error, String),

    #[error("Unexpected tool call {0:?}")]
    UnexpectedToolCall(Vec<ToolCall>),

    #[error("Cannot construct input without output")]
    InputRequired,

    #[error("Other error: {0}")]
    Other(String),
}
