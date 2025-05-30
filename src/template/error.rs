use serde_json::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Missing input variable: {0}")]
    MissingVariable(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] SerdeJsonError),

    #[error("Error: {0}")]
    OtherError(String),
}
