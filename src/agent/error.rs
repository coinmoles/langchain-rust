use thiserror::Error;

use crate::{chain::ChainError, llm::LLMError, template::TemplateError, tools::ToolError};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Prompt error: {0}")]
    PromptError(#[from] TemplateError),

    #[error("Tool error: {0}")]
    ToolError(#[from] ToolError),

    #[error("Too many consecutive fails: {0}")]
    TooManyConsecutiveFails(usize),

    #[error("Invalid response from LLM: {0}")]
    InvalidFormatError(String),

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
