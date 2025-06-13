use thiserror::Error;

use crate::{
    agent::AgentError, llm::LLMError, output_parser::OutputParseError, template::TemplateError,
};

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("LLM error: {0}")]
    LLMError(#[from] LLMError),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Retriever error: {0}")]
    RetrieverError(String),

    #[error("Output parse error: {0}")]
    OutputParseError(#[from] OutputParseError),

    #[error("Prompt error: {0}")]
    PromptError(#[from] TemplateError),

    #[error("Error: {0}")]
    OtherError(String),
}
