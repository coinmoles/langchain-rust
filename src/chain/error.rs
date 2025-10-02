use thiserror::Error;

use crate::agent::AgentError;
use crate::llm::LLMError;
use crate::template::TemplateError;
use crate::tools::{McpError, ToolError};
use crate::utils::parse::ParseError;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Retriever error: {0}")]
    RetrieverError(String),

    #[error("Output parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("Prompt error: {0}")]
    PromptError(#[from] TemplateError),

    #[error("Error: {0}")]
    OtherError(String),
}

impl<I> From<(I, ParseError)> for ChainError {
    fn from((_, err): (I, ParseError)) -> Self {
        ChainError::ParseError(err)
    }
}

impl From<McpError> for ChainError {
    fn from(err: McpError) -> Self {
        ChainError::AgentError(AgentError::ToolError(ToolError::McpError(Box::new(err))))
    }
}
