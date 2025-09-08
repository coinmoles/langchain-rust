use std::fmt::{self, Display};

#[derive(Clone)]
pub enum OpenAIModel {
    Gpt35,
    Gpt4,
    Gpt4Turbo,
    Gpt4o,
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
