use std::fmt::{self, Display};

use async_openai::types::responses::Usage;
use async_openai::types::{CompletionTokensDetails, CompletionUsage, PromptTokensDetails};
use indoc::writedoc;
use serde::{Deserialize, Serialize};

/// Token usage information.
///
/// Corresponds to [`CompletionUsage`](async_openai::types::CompletionUsage) for the chat
/// completions api and [`Usage`](async_openai::types::responses::Usage) for the responses api.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// The number of tokens in the prompt. a.k.a. input tokens.
    pub prompt_tokens: u32,
    /// The number of tokens in the completion. a.k.a. output tokens.
    pub completion_tokens: u32,
    /// The total number of tokens.
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Constructs a new [`TokenUsage`] with the given prompt and completion tokens.
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

impl TokenUsage {
    /// Merges two [`TokenUsage`] instances.
    pub fn merge(&self, other: &TokenUsage) -> Self {
        TokenUsage {
            prompt_tokens: self.prompt_tokens + other.prompt_tokens,
            completion_tokens: self.completion_tokens + other.completion_tokens,
            total_tokens: self.total_tokens + other.total_tokens,
        }
    }

    /// An helper function to merge multiple `Option<TokenUsage>` instances.
    ///
    /// If all options are `None`, returns `None`. Otherwise, returns `Some` with the merged usage.
    pub fn merge_options<'a>(
        usages: impl IntoIterator<Item = &'a Option<TokenUsage>>,
    ) -> Option<TokenUsage> {
        usages
            .into_iter()
            .fold(None, |acc, usage| match (acc, usage) {
                (Some(acc), Some(usage)) => Some(acc.merge(usage)),
                (Some(acc), None) => Some(acc),
                (None, Some(usage)) => Some(usage.clone()),
                (None, None) => None,
            })
    }
}

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

impl From<Usage> for TokenUsage {
    fn from(usage: Usage) -> Self {
        TokenUsage {
            prompt_tokens: usage.input_tokens,
            completion_tokens: usage.output_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

impl From<TokenUsage> for Usage {
    fn from(usage: TokenUsage) -> Self {
        Usage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            input_tokens_details: PromptTokensDetails {
                audio_tokens: None,
                cached_tokens: None,
            },
            output_tokens_details: CompletionTokensDetails {
                accepted_prediction_tokens: None,
                audio_tokens: None,
                reasoning_tokens: None,
                rejected_prediction_tokens: None,
            },
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

        assert_eq!(format!("{usage}"), expected_output);
    }
}
