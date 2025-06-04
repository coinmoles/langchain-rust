use serde::{de::DeserializeOwned, Serialize};

use crate::chain::ChainError;

pub trait ChainOutput: Serialize + DeserializeOwned + Clone + Send + Sync {
    fn try_from_string(s: impl Into<String>) -> Result<Self, ChainError> {
        let original: String = s.into();

        serde_json::from_str(&original).map_err(|_| ChainError::OutputFormatError(original))
    }
}

impl ChainOutput for String {
    fn try_from_string(s: impl Into<String>) -> Result<Self, ChainError> {
        Ok(s.into())
    }
}
