use async_openai::types::{ChatCompletionMessageToolCall, ChatCompletionToolType, FunctionCall};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
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

impl Serialize for ToolCall {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let openai_rep: Result<ChatCompletionMessageToolCall, _> = self.clone().try_into();

        if let Ok(tool_call) = openai_rep {
            return tool_call.serialize(serializer);
        }

        let function_call: Result<FunctionCall, _> = self.clone().try_into();
        if let Ok(function_call) = function_call {
            return function_call.serialize(serializer);
        }

        Err(serde::ser::Error::custom(
            "Failed to serialize ToolCall as ChatCompletionMessageToolCall or FunctionCall",
        ))
    }
}

impl<'de> Deserialize<'de> for ToolCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let openai_rep = ChatCompletionMessageToolCall::deserialize(deserializer)?;

        openai_rep.try_into().map_err(serde::de::Error::custom)
    }
}
