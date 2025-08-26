use crate::{
    agent::Strategy,
    chain::OutputCtor,
    schemas::{TokenUsage, WithUsage},
};

/// The final output of [`AgentExecutor`](crate::agent::AgentExecutor).
///
/// Contains content, token usage, and extra content based on the execution strategy.
pub struct ExecutionOutput<'input, O: OutputCtor, S: Strategy> {
    pub content: O::Target<'input>,
    pub extra_content: S::Output,
    pub usage: Option<TokenUsage>,
}

impl<'input, O: OutputCtor, S: Strategy> ExecutionOutput<'input, O, S> {
    /// Constructs a new [`ExecutionOutput`].
    pub fn new(
        content: O::Target<'input>,
        extra_content: S::Output,
        usage: Option<TokenUsage>,
    ) -> Self {
        Self {
            content,
            extra_content,
            usage,
        }
    }

    /// Removes strategy-specific extra contents, leaving only the content and the token usage.
    pub fn without_extra(self) -> WithUsage<O::Target<'input>> {
        WithUsage {
            content: self.content,
            usage: self.usage,
        }
    }
}
