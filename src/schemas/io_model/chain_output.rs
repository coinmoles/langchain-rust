use serde::Serialize;

pub trait ChainOutput<I>: Serialize + Clone + Send + Sync {
    fn try_from_string(input: I, s: impl Into<String>) -> Result<Self, TryFromStringError>;
}

impl<T> ChainOutput<T> for String {
    fn try_from_string(_input: T, s: impl Into<String>) -> Result<Self, TryFromStringError> {
        let original: String = s.into();
        Ok(original)
    }
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
