use std::io::{self, Write};
use std::pin::Pin;

use futures::Stream;
use serde_json::Value;

use crate::llm::LLMError;
use crate::schemas::TokenUsage;

/// LLM stream response with each chunk mapped to a [`LLMStreamChunk`].
///
/// Corresponds to
/// [`ChatCompletionResponseStream`](async_openai::types::ChatCompletionResponseStream) for the chat
/// completions api and [`ResponseStream`](async_openai::types::responses::ResponseStream) for the
/// responses api.
pub type LLMStream = Pin<Box<dyn Stream<Item = Result<LLMStreamChunk, LLMError>> + Send>>;

/// A chunk of data received from the LLM stream response.
///
/// Corresponds to
/// [`CreateChatCompletionStreamResponse`](async_openai::types::CreateChatCompletionStreamResponse)
/// for the chat completions api and
/// [`ResponseEvent`](async_openai::types::responses::ResponseEvent) for the responses api.
#[derive(Debug, Clone)]
pub struct LLMStreamChunk {
    /// The raw JSON value received from the LLM.
    pub value: Value,
    /// Token usage information, if available.
    pub tokens: Option<TokenUsage>,
    /// The text content of the chunk.
    pub content: String,
}

impl LLMStreamChunk {
    /// Constructs a new `LLMStreamChunk`.
    pub fn new<S: Into<String>>(value: Value, tokens: Option<TokenUsage>, content: S) -> Self {
        Self {
            value,
            tokens,
            content: content.into(),
        }
    }

    /// Writes the content of the chunk to standard output.
    pub fn to_stdout(&self) -> io::Result<()> {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{}", self.content)?;
        handle.flush()
    }
}
