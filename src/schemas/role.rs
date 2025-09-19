use std::fmt;

use serde::{Deserialize, Serialize};

/// The type of a message.
///
/// Corresponds to [`Role`](async_openai::types::responses::Role) for the responses api.
///
/// # Usage
/// ```rust,ignore
/// let system_message_type = MessageType::SystemMessage;
/// let ai_message_type = MessageType::AIMessage;
/// let human_message_type = MessageType::HumanMessage;
/// ```
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "ai")]
    Ai,
    #[serde(rename = "human")]
    Human,
    #[serde(rename = "tool")]
    Tool,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::Ai => write!(f, "ai"),
            Role::Human => write!(f, "human"),
            Role::Tool => write!(f, "tool"),
        }
    }
}
