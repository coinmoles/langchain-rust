use async_openai::types::{
    responses::{Function, ToolDefinition},
    ChatCompletionTool, FunctionObject,
};
use serde_json::Value;

use crate::tools::McpTool;

pub struct ToolSpec<'a> {
    pub functions: &'a [FunctionSpec],
    pub mcps: &'a [McpTool],
}

/// A struct representing the tool definition payload.
///
/// While `async_openai` provides structs for this,
/// two separate structs exist for the responses api ([`Function`](async_openai::types::responses::Function))
/// and the chat completions api [`FunctionObject`](async_openai::types::FunctionObject)).
/// This struct provides a unified api for internal use with easy conversion into the two structs.
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Value,
    pub strict: bool,
}

impl FunctionSpec {
    pub fn new(name: String, description: Option<String>, parameters: Value, strict: bool) -> Self {
        Self {
            name,
            description,
            parameters,
            strict,
        }
    }
}

impl From<Function> for FunctionSpec {
    fn from(function: Function) -> Self {
        FunctionSpec {
            name: function.name,
            description: function.description,
            parameters: function.parameters,
            strict: function.strict,
        }
    }
}

impl From<FunctionSpec> for Function {
    fn from(tool: FunctionSpec) -> Self {
        Function {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters,
            strict: tool.strict,
        }
    }
}

impl From<FunctionSpec> for ToolDefinition {
    fn from(tool: FunctionSpec) -> Self {
        ToolDefinition::Function(Function::from(tool))
    }
}

impl From<FunctionObject> for FunctionSpec {
    fn from(function: FunctionObject) -> Self {
        FunctionSpec {
            name: function.name,
            description: function.description,
            parameters: function.parameters.unwrap_or(Value::Null),
            strict: function.strict.unwrap_or_default(),
        }
    }
}

impl From<FunctionSpec> for FunctionObject {
    fn from(tool: FunctionSpec) -> Self {
        FunctionObject {
            name: tool.name,
            description: tool.description,
            parameters: Some(tool.parameters),
            strict: Some(tool.strict),
        }
    }
}

impl From<FunctionSpec> for ChatCompletionTool {
    fn from(tool: FunctionSpec) -> Self {
        ChatCompletionTool {
            r#type: async_openai::types::ChatCompletionToolType::Function,
            function: FunctionObject::from(tool),
        }
    }
}
