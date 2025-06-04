use super::TokenUsage;

#[derive(Debug)]
pub struct WithUsage<O> {
    pub content: O,
    pub usage: Option<TokenUsage>,
}

pub trait IntoWithUsage<T> {
    fn with_usage(self, usage: Option<TokenUsage>) -> WithUsage<T>;
}

impl<T> IntoWithUsage<T> for T {
    fn with_usage(self, usage: Option<TokenUsage>) -> WithUsage<T> {
        WithUsage {
            content: self,
            usage,
        }
    }
}
