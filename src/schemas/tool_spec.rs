use async_openai::types::{responses::Function, ChatCompletionTool, FunctionObject};
use serde_json::Value;

/// A struct representing the tool definition payload.
///
/// While `async_openai` provides structs for this,
/// two separate structs exist for the responses api ([`Function`](async_openai::types::responses::Function))
/// and the chat completions api [`FunctionObject`](async_openai::types::FunctionObject)).
/// This struct provides a unified api for internal use with easy conversion into the two structs.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
    pub strict: bool,
}

impl ToolSpec {
    pub fn new(name: String, description: Option<String>, parameters: Value, strict: bool) -> Self {
        Self {
            name,
            description,
            parameters,
            strict,
        }
    }
}

impl From<Function> for ToolSpec {
    fn from(function: Function) -> Self {
        ToolSpec {
            name: function.name,
            description: function.description,
            parameters: function.parameters,
            strict: function.strict,
        }
    }
}

impl From<ToolSpec> for Function {
    fn from(tool: ToolSpec) -> Self {
        Function {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters,
            strict: tool.strict,
        }
    }
}

impl From<ToolSpec> for async_openai::types::responses::ToolDefinition {
    fn from(tool: ToolSpec) -> Self {
        async_openai::types::responses::ToolDefinition::Function(Function::from(tool))
    }
}

impl From<FunctionObject> for ToolSpec {
    fn from(function: FunctionObject) -> Self {
        ToolSpec {
            name: function.name,
            description: function.description,
            parameters: function.parameters.unwrap_or(Value::Null),
            strict: function.strict.unwrap_or_default(),
        }
    }
}

impl From<ToolSpec> for FunctionObject {
    fn from(tool: ToolSpec) -> Self {
        FunctionObject {
            name: tool.name,
            description: tool.description,
            parameters: Some(tool.parameters),
            strict: Some(tool.strict),
        }
    }
}

impl From<ToolSpec> for ChatCompletionTool {
    fn from(tool: ToolSpec) -> Self {
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: FunctionObject::from(tool),
        }
    }
}
