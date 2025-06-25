use std::fmt::{self, Display};

use async_openai::types::{ChatCompletionResponseMessage, Role};
use macros::Ctor;
use serde::{Deserialize, Serialize};

use crate::{chain::ChainOutput, llm::LLMError, schemas::ToolCall};

#[derive(Debug, Clone, Ctor)]
pub enum LLMOutput {
    Text(String),
    ToolCall(Vec<ToolCall>),
}

impl LLMOutput {
    pub fn into_text(self) -> Result<String, serde_json::Error> {
        let text = match self {
            LLMOutput::Text(text) => text,
            LLMOutput::ToolCall(t) => serde_json::to_string_pretty(&t)?,
        };
        Ok(text)
    }
}

impl Default for LLMOutput {
    fn default() -> Self {
        LLMOutput::Text("".into())
    }
}

impl<T> ChainOutput<T> for LLMOutput {
    fn construct_from_text(
        text: impl Into<String>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        Ok(LLMOutput::Text(text.into()))
    }

    fn construct_from_tool_call(
        tool_calls: Vec<ToolCall>,
    ) -> Result<Self, crate::output_parser::OutputParseError> {
        Ok(LLMOutput::ToolCall(tool_calls))
    }
}

// Convert to async-openai type
impl TryFrom<ChatCompletionResponseMessage> for LLMOutput {
    type Error = LLMError;

    fn try_from(value: ChatCompletionResponseMessage) -> Result<Self, Self::Error> {
        if let Some(tool_calls) = value.tool_calls {
            if !tool_calls.is_empty() {
                return Ok(LLMOutput::ToolCall(
                    tool_calls
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()?,
                ));
            }
        }
        #[allow(deprecated)]
        if let Some(function_call) = value.function_call {
            return Ok(LLMOutput::ToolCall(vec![function_call.try_into()?]));
        }
        if let Some(content) = value.content {
            return Ok(LLMOutput::Text(content));
        }
        if let Some(refusal) = value.refusal {
            return Err(LLMError::Refused(refusal));
        }
        // TODO: Add other cases (Audio, etc.)
        Err(LLMError::OtherError(
            "Cannot convert LLM generation result to LLMOutput".into(),
        ))
    }
}

impl TryFrom<LLMOutput> for ChatCompletionResponseMessage {
    type Error = serde_json::Error;

    fn try_from(value: LLMOutput) -> Result<Self, Self::Error> {
        #[allow(deprecated)]
        match value {
            LLMOutput::Text(text) => Ok(ChatCompletionResponseMessage {
                content: Some(text),
                refusal: None,
                role: Role::Assistant,
                audio: None,
                tool_calls: None,
                function_call: None,
            }),
            LLMOutput::ToolCall(tool_calls) => Ok(ChatCompletionResponseMessage {
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

impl Display for LLMOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMOutput::Text(text) => write!(f, "{}", text),
            LLMOutput::ToolCall(tool_calls) => {
                writeln!(f, "Structured tool call:")?;
                for tool_call in tool_calls {
                    writeln!(f, "{}", tool_call)?;
                }
                Ok(())
            }
        }
    }
}
