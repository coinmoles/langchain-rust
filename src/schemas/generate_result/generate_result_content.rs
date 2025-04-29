use std::fmt::{self, Display};

use async_openai::types::{ChatCompletionResponseMessage, Role};
use serde::{de::Error, Deserialize, Serialize};

use super::ToolCall;

#[derive(Debug, Clone)]
pub enum GenerateResultContent {
    Text(String),
    ToolCall(Vec<ToolCall>),
    Refusal(String),
}

impl GenerateResultContent {
    pub fn text(&self) -> &str {
        match self {
            GenerateResultContent::Text(text) => text,
            GenerateResultContent::ToolCall(_) => "",
            GenerateResultContent::Refusal(refusal) => refusal,
        }
    }
}

impl Default for GenerateResultContent {
    fn default() -> Self {
        GenerateResultContent::Text(String::new())
    }
}

// Convert to async-openai type
impl TryFrom<ChatCompletionResponseMessage> for GenerateResultContent {
    type Error = serde_json::Error;

    fn try_from(value: ChatCompletionResponseMessage) -> Result<Self, Self::Error> {
        #[allow(deprecated)]
        if let Some(tool_calls) = value.tool_calls {
            Ok(GenerateResultContent::ToolCall(
                tool_calls
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?,
            ))
        } else if let Some(function_call) = value.function_call {
            Ok(GenerateResultContent::ToolCall(vec![
                function_call.try_into()?
            ]))
        } else if let Some(content) = value.content {
            Ok(GenerateResultContent::Text(content))
        } else if let Some(refusal) = value.refusal {
            Ok(GenerateResultContent::Refusal(refusal))
        } else {
            // TODO: Add other cases (Audio, etc.)
            Err(serde_json::Error::custom(
                "Can't convert to GenerateResultContent",
            ))
        }
    }
}

impl TryFrom<GenerateResultContent> for ChatCompletionResponseMessage {
    type Error = serde_json::Error;

    fn try_from(value: GenerateResultContent) -> Result<Self, Self::Error> {
        #[allow(deprecated)]
        match value {
            GenerateResultContent::Text(text) => Ok(ChatCompletionResponseMessage {
                content: Some(text),
                refusal: None,
                role: Role::Assistant,
                audio: None,
                tool_calls: None,
                function_call: None,
            }),
            GenerateResultContent::ToolCall(tool_calls) => Ok(ChatCompletionResponseMessage {
                content: None,
                refusal: None,
                role: Role::Assistant,
                audio: None,
                tool_calls: Some(
                    tool_calls
                        .into_iter()
                        .map(ToolCall::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                function_call: None,
            }),
            GenerateResultContent::Refusal(refusal) => Ok(ChatCompletionResponseMessage {
                content: None,
                refusal: Some(refusal),
                role: Role::Assistant,
                audio: None,
                tool_calls: None,
                function_call: None,
            }),
        }
    }
}

impl Serialize for GenerateResultContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let openai_rep: ChatCompletionResponseMessage =
            self.clone().try_into().map_err(serde::ser::Error::custom)?;

        openai_rep.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GenerateResultContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let openai_rep = ChatCompletionResponseMessage::deserialize(deserializer)?;

        openai_rep.try_into().map_err(serde::de::Error::custom)
    }
}

impl Display for GenerateResultContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateResultContent::Text(text) => write!(f, "{}", text),
            GenerateResultContent::ToolCall(tool_calls) => {
                writeln!(f, "Structured tool call:")?;
                for tool_call in tool_calls {
                    writeln!(f, "{}", tool_call)?;
                }
                Ok(())
            }
            GenerateResultContent::Refusal(refusal) => write!(f, "Refused: {}", refusal),
        }
    }
}
