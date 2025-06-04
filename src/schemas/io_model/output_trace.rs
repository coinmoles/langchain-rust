use serde::Serialize;
use serde_json::Value;

use super::{TokenUsage, WithUsage};

pub struct OutputTrace<T> {
    pub previous_steps: Vec<WithUsage<Value>>,
    pub final_step: WithUsage<T>,
    pub total_usage: Option<TokenUsage>,
}

impl<T> OutputTrace<T> {
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

    pub fn single(step: WithUsage<T>) -> Self {
        let total_usage = step.usage.clone();

        OutputTrace {
            previous_steps: Vec::new(),
            final_step: step,
            total_usage,
        }
    }

    pub fn extend<T2>(self, other: OutputTrace<T2>) -> Result<OutputTrace<T2>, serde_json::Error>
    where
        T: Serialize,
    {
        let total_usage = TokenUsage::merge_options([&self.total_usage, &other.total_usage]);

        let steps = self
            .previous_steps
            .into_iter()
            .chain(std::iter::once(WithUsage {
                content: serde_json::to_value(self.final_step.content)?,
                usage: self.final_step.usage,
            }))
            .chain(other.previous_steps)
            .collect();

        Ok(OutputTrace {
            previous_steps: steps,
            final_step: other.final_step,
            total_usage,
        })
    }
}
