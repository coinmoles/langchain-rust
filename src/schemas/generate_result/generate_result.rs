use super::{GenerateResultContent, TokenUsage};

#[derive(Debug, Clone, Default)]
pub struct GenerateResult {
    pub content: GenerateResultContent,
    pub usage: Option<TokenUsage>,
}

impl GenerateResult {
    pub fn new(content: GenerateResultContent, usage: Option<TokenUsage>) -> Self {
        Self { content, usage }
    }
}
