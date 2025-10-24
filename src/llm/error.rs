use async_openai::error::OpenAIError;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use thiserror::Error;

use crate::utils::parse::ParseError;

#[derive(Error, Debug)]
pub enum LLMError {
    /// Error from the [`async_openai`].
    #[error("OpenAI error: {0:?}")]
    OpenAIError(Box<OpenAIError>),

    /// Error from the [`reqwest`].
    #[error("Network request failed: {0:?}")]
    RequestError(#[from] ReqwestError),

    /// Error parsing LLM output.
    ///
    /// Returned when the response payload is valid but its content cannot be parsed into the
    /// expected format.
    #[error("Failed to parse LLM output: {0}")]
    ParseError(#[from] ParseError),

    /// Error indicating that the response payload cannot be deserialized into the expected
    /// schema. Usually indicates the provider returned an unexpected response.
    #[error("Failed to serialize/deserialize response: {0:?}")]
    ResponseSerdeError(SerdeJsonError),

    #[error("Invalid header value for the header `{0}`: {1:?}")]
    InvalidHeader(String, String),

    /// Error indicating that the LLM response does not contain the expected content.
    #[error("Content not found in response: Expected at {0}")]
    ContentNotFound(String),

    /// Error from the LLM refusing to answer a prompt.
    #[error("LLM refused to answer: {0}")]
    Refused(String),

    /// Error not covered by other variants.
    #[error("{0}")]
    Other(String),
}

impl LLMError {
    pub fn invalid_header(header: impl Into<String>, value: impl Into<String>) -> Self {
        LLMError::InvalidHeader(header.into(), value.into())
    }

    /// Constructs a new `LLMError::ContentNotFound` with the given message.
    pub fn content_not_found(msg: impl Into<String>) -> Self {
        LLMError::ContentNotFound(msg.into())
    }

    /// Create a new `LLMError::OtherError` with the given message.
    pub fn other(msg: impl Into<String>) -> Self {
        LLMError::Other(msg.into())
    }
}

impl From<OpenAIError> for LLMError {
    fn from(value: OpenAIError) -> Self {
        LLMError::OpenAIError(Box::new(value))
    }
}
