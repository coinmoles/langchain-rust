use futures::Stream;
use serde_json::Value;
use std::{
    io::{self, Write},
    pin::Pin,
};

use crate::{llm::LLMError, schemas::TokenUsage};

pub type LLMStream = Pin<Box<dyn Stream<Item = Result<LLMStreamChunk, LLMError>> + Send>>;

#[derive(Debug, Clone)]
pub struct LLMStreamChunk {
    pub value: Value,
    pub tokens: Option<TokenUsage>,
    pub content: String,
}

impl LLMStreamChunk {
    pub fn new<S: Into<String>>(value: Value, tokens: Option<TokenUsage>, content: S) -> Self {
        Self {
            value,
            tokens,
            content: content.into(),
        }
    }

    pub fn to_stdout(&self) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{}", self.content)?;
        handle.flush()
    }
}
