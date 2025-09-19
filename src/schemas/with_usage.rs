use super::TokenUsage;

/// An output with the token usage information.
///
/// # Fields
/// - `content`: The actual output content.
/// - `usage`: The token usage information.
#[derive(Debug)]
pub struct WithUsage<O> {
    /// The actual output content.
    pub content: O,
    /// The token usage information.
    pub usage: Option<TokenUsage>,
}

/// A trait to convert a value into a `WithUsage`.
///
/// The trait is implemented for all types.
pub trait IntoWithUsage<T> {
    /// Wraps the value with the given token usage information.
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
