use std::fmt::{self, Display};

use async_openai::types::responses::{FunctionCall as ResponsesFunctionCall, OutputStatus};
use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionToolType, FunctionCall};
use indoc::indoc;
use serde_json::Value;
use uuid::Uuid;

use crate::utils::helper::add_indent;

/// A tool call made by an LLM.
///
/// Corresponds to
/// [`ChatCompletionMessageToolCall`](async_openai::types::ChatCompletionMessageToolCall) /
/// [`FunctionCall`](async_openai::types::FunctionCall) for the chat completions api and
/// [`FunctionCall`](async_openai::types::responses::FunctionCall) for the responses api.
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// The id of the tool call.
    pub id: String,
    /// The name of the tool to call.
    pub name: String,
    /// The arguments to pass to the tool.
    pub arguments: Value,
}

impl ToolCall {
    /// Constructs a new [`ToolCall`].
    ///
    /// If `id` is not provided, a new UUID will be generated.
    pub fn new(id: Option<String>, name: impl Into<String>, arguments: Option<Value>) -> Self {
        Self {
            id: id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: name.into(),
            arguments: arguments.unwrap_or(Value::Null),
        }
    }
}

impl TryFrom<ChatCompletionMessageToolCall> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(value: ChatCompletionMessageToolCall) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.function.name,
            arguments: serde_json::from_str(&value.function.arguments)?,
        })
    }
}

impl From<ToolCall> for ChatCompletionMessageToolCall {
    fn from(value: ToolCall) -> Self {
        Self {
            id: value.id,
            r#type: ChatCompletionToolType::Function,
            function: FunctionCall {
                name: value.name,
                arguments: serde_json::to_string(&value.arguments)
                    .unwrap_or_else(|_| value.arguments.to_string()),
            },
        }
    }
}

impl TryFrom<FunctionCall> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(value: FunctionCall) -> Result<Self, Self::Error> {
        Ok(Self {
            id: String::new(),
            name: value.name,
            arguments: serde_json::from_str(&value.arguments)?,
        })
    }
}

impl From<ToolCall> for FunctionCall {
    fn from(value: ToolCall) -> Self {
        Self {
            name: value.name,
            arguments: serde_json::to_string(&value.arguments)
                .unwrap_or_else(|_| value.arguments.to_string()),
        }
    }
}

impl TryFrom<ResponsesFunctionCall> for ToolCall {
    type Error = serde_json::Error;

    fn try_from(value: ResponsesFunctionCall) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            name: value.name,
            arguments: serde_json::from_str(&value.arguments)?,
        })
    }
}

impl From<ToolCall> for ResponsesFunctionCall {
    fn from(value: ToolCall) -> Self {
        Self {
            id: value.id,
            name: value.name,
            arguments: serde_json::to_string(&value.arguments)
                .unwrap_or_else(|_| value.arguments.to_string()),
            // TODO: better handle these fields.
            call_id: String::new(),
            status: OutputStatus::Completed,
        }
    }
}

impl Display for ToolCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            indoc! {r#"
            {{ 
                "action": "{}", 
                "action_input": {} 
            }}"#},
            self.name,
            add_indent(
                &serde_json::to_string_pretty(&self.arguments)
                    .unwrap_or_else(|_| self.arguments.to_string()),
                4,
                false
            )
        )
    }
}
