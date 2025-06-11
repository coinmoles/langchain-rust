use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum `MessageType` represents the type of a message.
/// It can be a `SystemMessage`, `AIMessage`, or `HumanMessage`.
///
/// # Usage
/// ```rust,ignore
/// let system_message_type = MessageType::SystemMessage;
/// let ai_message_type = MessageType::AIMessage;
/// let human_message_type = MessageType::HumanMessage;
/// ```
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "ai")]
    Ai,
    #[serde(rename = "human")]
    Human,
    #[serde(rename = "tool")]
    Tool,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::System
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::System => write!(f, "system"),
            MessageType::Ai => write!(f, "ai"),
            MessageType::Human => write!(f, "human"),
            MessageType::Tool => write!(f, "tool"),
        }
    }
}
