use thiserror::Error;

use crate::schemas::ToolCall;

#[derive(Debug, Error)]
pub enum OutputParseError {
    #[error("Deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("Unexpected tool call {0:?}")]
    UnexpectedToolCall(Vec<ToolCall>),

    #[error("Cannot construct input without output")]
    InputRequired,

    #[error("Other error: {0}")]
    Other(String),
}
