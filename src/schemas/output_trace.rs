use serde::Serialize;
use serde_json::Value;

use crate::schemas::{TokenUsage, WithUsage};

/// An output trace of a sequential chain.
///
/// # Fields
/// - [`previous_steps`](Self::previous_steps): The previous steps in the chain with the token usage
///   for each step.
/// - [`final_step`](Self::final_step): The final output and its token usage.
/// - [`total_usage`](Self::total_usage): The total token usage across all steps.
pub struct OutputTrace<T> {
    /// The previous steps in the chain with the token usage for each step.
    pub previous_steps: Vec<WithUsage<Value>>,
    /// The final output and its token usage.
    pub final_step: WithUsage<T>,
    /// The total token usage across all steps.
    pub total_usage: Option<TokenUsage>,
}

impl<T> OutputTrace<T> {
    /// Constructs a new [`OutputTrace`].
    pub fn new(previous_steps: Vec<WithUsage<Value>>, final_step: WithUsage<T>) -> Self {
        let total_usage = TokenUsage::merge_options(
            previous_steps
                .iter()
                .map(|s| &s.usage)
                .chain(std::iter::once(&final_step.usage)),
        );

        OutputTrace {
            previous_steps,
            final_step,
            total_usage,
        }
    }

    /// Constructs a new [`OutputTrace`] with a single step.
    pub fn single(step: WithUsage<T>) -> Self {
        let total_usage = step.usage.clone();

        OutputTrace {
            previous_steps: Vec::new(),
            final_step: step,
            total_usage,
        }
    }

    /// Extends the current [`OutputTrace`] with another [`OutputTrace`], merging their steps and
    /// token usage.
    pub fn extend<T2>(self, other: OutputTrace<T2>) -> OutputTrace<T2>
    where
        T: Serialize,
    {
        let total_usage = TokenUsage::merge_options([&self.total_usage, &other.total_usage]);

        let steps = self
            .previous_steps
            .into_iter()
            .chain(std::iter::once(WithUsage {
                content: serde_json::to_value(self.final_step.content)
                    .unwrap_or(Value::String("Failed to serialize step".into())),
                usage: self.final_step.usage,
            }))
            .chain(other.previous_steps)
            .collect();

        OutputTrace {
            previous_steps: steps,
            final_step: other.final_step,
            total_usage,
        }
    }
}
