use std::fmt::{self, Display};

use async_openai::types::{ChatCompletionResponseMessage, Role};
use macros::Ctor;
use serde::{Deserialize, Serialize};

use crate::chain::ChainOutput;
use crate::llm::LLMError;
use crate::schemas::ToolCall;

#[derive(Debug, Clone, Ctor)]
pub struct LLMOutput {
    pub thought: Option<String>,
    pub event: LLMEvent,
}

#[derive(Debug, Clone)]
pub enum LLMEvent {
    Text(String),
    ToolCall(Vec<ToolCall>),
}

impl LLMEvent {
    pub fn into_text(self) -> Result<String, serde_json::Error> {
        let text = match self {
            LLMEvent::Text(text) => text,
            LLMEvent::ToolCall(tool_calls) => tool_calls
                .iter()
                .map(|tool_call| tool_call.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        Ok(text)
    }
}

impl Default for LLMEvent {
    fn default() -> Self {
        LLMEvent::Text("".into())
    }
}

impl<T> ChainOutput<T> for LLMOutput {
    fn from_text(text: impl Into<String>) -> Result<Self, crate::output_parser::OutputParseError> {
        let event = LLMEvent::Text(text.into());
        Ok(LLMOutput {
            thought: None,
            event,
        })
    }

    fn from_tool_call(
        thought: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        let event = LLMEvent::ToolCall(tool_calls);
        Ok(LLMOutput { thought, event })
    }
}

// Convert to async-openai type
impl TryFrom<ChatCompletionResponseMessage> for LLMOutput {
    type Error = LLMError;

    fn try_from(value: ChatCompletionResponseMessage) -> Result<Self, Self::Error> {
        if let Some(tool_calls) = value.tool_calls {
            if !tool_calls.is_empty() {
                let tool_calls = tool_calls
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(LLMOutput {
                    thought: value.content,
                    event: LLMEvent::ToolCall(tool_calls),
                });
            }
        }
        #[allow(deprecated)]
        if let Some(function_call) = value.function_call {
            return Ok(LLMOutput {
                thought: value.content,
                event: LLMEvent::ToolCall(vec![function_call.try_into()?]),
            });
        }
        if let Some(content) = value.content {
            return Ok(LLMOutput {
                thought: None,
                event: LLMEvent::Text(content),
            });
        }
        if let Some(refusal) = value.refusal {
            return Err(LLMError::Refused(refusal));
        }
        // TODO: Add other cases (Audio, etc.)
        Err(LLMError::other(
            "Cannot convert LLM generation result to LLMOutput",
        ))
    }
}

impl TryFrom<LLMOutput> for ChatCompletionResponseMessage {
    type Error = serde_json::Error;

    fn try_from(value: LLMOutput) -> Result<Self, Self::Error> {
        #[allow(deprecated)]
        match value.event {
            LLMEvent::Text(text) => Ok(ChatCompletionResponseMessage {
                content: Some(text),
                refusal: None,
                role: Role::Assistant,
                audio: None,
                tool_calls: None,
                function_call: None,
            }),
            LLMEvent::ToolCall(tool_calls) => Ok(ChatCompletionResponseMessage {
                content: value.thought,
                refusal: None,
                role: Role::Assistant,
                audio: None,
                tool_calls: Some(tool_calls.into_iter().map(Into::into).collect::<Vec<_>>()),
                function_call: None,
            }),
        }
    }
}

impl Serialize for LLMOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let openai_rep: ChatCompletionResponseMessage =
            self.clone().try_into().map_err(serde::ser::Error::custom)?;

        openai_rep.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LLMOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let openai_rep = ChatCompletionResponseMessage::deserialize(deserializer)?;

        openai_rep.try_into().map_err(serde::de::Error::custom)
    }
}

impl Display for LLMEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMEvent::Text(text) => write!(f, "{text}"),
            LLMEvent::ToolCall(tool_calls) => {
                for (i, tool_call) in tool_calls.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{tool_call}")?;
                }
                Ok(())
            }
        }
    }
}

impl Display for LLMOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(thought) = &self.thought {
            writeln!(f, "{thought}")?;
        }
        match &self.event {
            LLMEvent::Text(text) => {
                if self.thought.is_some() {
                    writeln!(f)?;
                }
                write!(f, "{text}")?;
            }
            LLMEvent::ToolCall(tool_calls) => {
                if tool_calls.is_empty() {
                    return Ok(());
                }

                writeln!(f, "```json")?;
                for (i, tool_call) in tool_calls.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{tool_call}")?;
                }
                write!(f, "\n```")?;
            }
        }
        Ok(())
    }
}
