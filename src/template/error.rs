use thiserror::Error;

/// Errors that can occur during template processing.
#[derive(Error, Debug)]
pub enum TemplateError {
    /// Error raised when required input variable is missing.
    #[error("Missing input variable: {0}")]
    MissingVariable(String),

    /// Error not covered by other variants.
    #[error("Error: {0}")]
    OtherError(String),
}
