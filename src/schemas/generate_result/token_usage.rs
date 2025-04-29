use std::fmt::{self, Display};

use async_openai::types::CompletionUsage;
use indoc::writedoc;
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

impl TokenUsage {
    pub fn merge(&self, other: &TokenUsage) -> Self {
        TokenUsage {
            prompt_tokens: self.prompt_tokens + other.prompt_tokens,
            completion_tokens: self.completion_tokens + other.completion_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }

    pub fn merge_options(
        usage1: Option<TokenUsage>,
        usage2: Option<TokenUsage>,
    ) -> Option<TokenUsage> {
        match (usage1, usage2) {
            (Some(usage1), Some(usage2)) => Some(usage1.merge(&usage2)),
            (Some(usage), None) => Some(usage),
            (None, Some(usage)) => Some(usage),
            (None, None) => None,
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

impl Display for TokenUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writedoc! {
        f,
        "
        - Prompt Tokens: {}
        - Completion Tokens: {}
        - Total Tokens: {}",
        self.prompt_tokens, self.completion_tokens, self.total_tokens}
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn test_display() {
        let usage = TokenUsage::new(10, 20);
        let expected_output = indoc! {"
            - Prompt Tokens: 10
            - Completion Tokens: 20
            - Total Tokens: 30"};

        assert_eq!(format!("{}", usage), expected_output);
    }
}
