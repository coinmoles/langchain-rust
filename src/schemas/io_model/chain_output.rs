pub use macros::ChainOutput;

use serde::Serialize;

pub trait ChainOutput<I>: Serialize + Clone + Send + Sync {
    fn parse_output(input: I, response: impl Into<String>) -> Result<Self, OutputParseError>;
}

impl<T> ChainOutput<T> for String {
    fn parse_output(_input: T, output: impl Into<String>) -> Result<Self, OutputParseError> {
        let original: String = output.into();
        Ok(original)
    }
}

#[derive(Debug)]
pub struct OutputParseError {
    pub original: String,
    pub error: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl std::fmt::Display for OutputParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(err) = &self.error {
            writeln!(f, "{err}")?;
        }
        write!(f, "Original response:\n{}", self.original)
    }
}

impl std::error::Error for OutputParseError {}
