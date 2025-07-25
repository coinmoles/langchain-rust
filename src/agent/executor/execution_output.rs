use crate::{
    agent::Strategy,
    chain::OutputCtor,
    schemas::{TokenUsage, WithUsage},
};

pub struct ExecutionOutput<'input, O: OutputCtor, S: Strategy> {
    pub content: O::Target<'input>,
    pub extra_content: S::Output,
    pub usage: Option<TokenUsage>,
}

impl<'input, O: OutputCtor, S: Strategy> ExecutionOutput<'input, O, S> {
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

    pub fn without_extra(self) -> WithUsage<O::Target<'input>> {
        WithUsage {
            content: self.content,
            usage: self.usage,
        }
    }
}
