use std::fmt::{self, Display};

/// Helper enum for OpenAI model names.
#[derive(Clone)]
pub enum OpenAIModel {
    /// `"gpt-3.5-turbo"`
    Gpt35,
    /// `"gpt-4"`
    Gpt4,
    /// `"gpt-4-turbo-preview"`
    Gpt4Turbo,
    /// `"gpt-4o"`
    Gpt4o,
    /// `"gpt-4o-mini"`
    Gpt4oMini,
}

impl Display for OpenAIModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpenAIModel::Gpt35 => write!(f, "gpt-3.5-turbo"),
            OpenAIModel::Gpt4 => write!(f, "gpt-4"),
            OpenAIModel::Gpt4Turbo => write!(f, "gpt-4-turbo-preview"),
            OpenAIModel::Gpt4o => write!(f, "gpt-4o"),
            OpenAIModel::Gpt4oMini => write!(f, "gpt-4o-mini"),
        }
    }
}

impl From<OpenAIModel> for String {
    fn from(val: OpenAIModel) -> Self {
        val.to_string()
    }
}
