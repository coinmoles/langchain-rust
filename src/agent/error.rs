use thiserror::Error;

use crate::{chain::ChainError, llm::LLMError, template::TemplateError, tools::ToolError};

/// Errors that can occur during agent operations.
#[derive(Error, Debug)]
pub enum AgentError {
    /// An error that occurred during interaction with the LLM.
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    /// An error that occurred while formatting the prompt, e.g. missing variables or invalid templates.
    #[error("Prompt error: {0}")]
    PromptError(#[from] TemplateError),

    /// An error that occurred during tool invocation.
    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),

    /// An error indicating that the agent has failed repeatedly and exceeded the allowed failure threshold.
    #[error("Too many consecutive fails: {0}")]
    TooManyConsecutiveFails(usize),

    /// An error that occurs when the LLM response could not be parsed or did not conform to the expected format.
    #[error("Invalid response from LLM: {0}")]
    InvalidFormatError(String),

    /// A catch-all variant for miscellaneous agent errors.
    #[error("Error: {0}")]
    OtherError(String),
}

impl From<ChainError> for AgentError {
    fn from(error: ChainError) -> Self {
        match error {
            ChainError::AgentError(e) => e,
            ChainError::LLMError(e) => AgentError::LLMError(e),
            ChainError::PromptError(e) => AgentError::PromptError(e),
            ChainError::OtherError(msg) => AgentError::OtherError(msg),
            other => AgentError::OtherError(other.to_string()),
        }
    }
}
