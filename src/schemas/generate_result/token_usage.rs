use async_openai::types::CompletionUsage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    // TODO: add details
}

impl TokenUsage {
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

// Convert from async-openai type
impl From<CompletionUsage> for TokenUsage {
    fn from(usage: CompletionUsage) -> Self {
        TokenUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

impl From<TokenUsage> for CompletionUsage {
    fn from(usage: TokenUsage) -> Self {
        CompletionUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
        }
    }
}
