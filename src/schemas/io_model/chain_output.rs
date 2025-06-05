pub use macros::AsInput;
use serde::Serialize;

use crate::schemas::ChainInput;

pub trait ChainOutput: Serialize + Clone + Send + Sync {
    fn try_from_string(s: impl Into<String>) -> Result<Self, TryFromStringError>;
}

impl ChainOutput for String {
    fn try_from_string(s: impl Into<String>) -> Result<Self, TryFromStringError> {
        Ok(s.into())
    }
}
pub trait AsInput {
    type AsInput<'a>: ChainInput
    where
        Self: 'a;

    fn as_input(&self) -> Self::AsInput<'_>;
}

#[derive(Debug)]
pub struct TryFromStringError(pub String);

impl std::fmt::Display for TryFromStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to construct desired struct from output:\n{}",
            self.0
        )
    }
}

impl std::error::Error for TryFromStringError {}
