use std::fmt::{self, Display};

use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionToolType, FunctionCall};
use indoc::indoc;
use serde_json::Value;
use uuid::Uuid;

use crate::utils::helper::add_indent;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

impl ToolCall {
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

impl TryFrom<ToolCall> for ChatCompletionMessageToolCall {
    type Error = serde_json::Error;

    fn try_from(value: ToolCall) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            r#type: ChatCompletionToolType::Function,
            function: FunctionCall {
                name: value.name,
                arguments: serde_json::to_string(&value.arguments)?,
            },
        })
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

impl TryFrom<ToolCall> for FunctionCall {
    type Error = serde_json::Error;

    fn try_from(value: ToolCall) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            arguments: serde_json::to_string(&value.arguments)?,
        })
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
