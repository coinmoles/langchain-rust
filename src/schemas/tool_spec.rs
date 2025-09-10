use async_openai::types::{
    responses::{Function, ToolDefinition},
    ChatCompletionTool, FunctionObject,
};
use indoc::formatdoc;
use schemars::{schema_for, Schema};
use serde_json::json;

use crate::{
    tools::{describe_parameters, EmptyFunctionInput, McpTool},
    utils::helper::normalize_tool_name,
};

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
    pub parameters: Schema,
    pub strict: bool,
}

impl FunctionSpec {
    pub fn new(
        name: String,
        description: Option<String>,
        parameters: Schema,
        strict: bool,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            strict,
        }
    }

    pub fn as_json(&self) -> String {
        let json = json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
            "strict": self.strict,
        });
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    }

    pub fn describe(&self) -> String {
        let name = normalize_tool_name(&self.name);
        let desc = self.description.as_deref().unwrap_or("");
        let parameters = describe_parameters(&self.parameters);

        match parameters {
            Ok(parameters) => formatdoc! {"
                > {name}: {desc}
                <INPUT_FORMAT>
                {parameters}
                </INPUT_FORMAT>"},
            Err(e) => {
                log::warn!("Failed to describe parameters for tool {}: {e}", self.name);
                format!("> {name}: {desc}")
            }
        }
    }
}

impl TryFrom<Function> for FunctionSpec {
    type Error = serde_json::Error;

    fn try_from(function: Function) -> Result<Self, Self::Error> {
        let parameters = Schema::try_from(function.parameters)?;
        let spec = FunctionSpec {
            name: function.name,
            description: function.description,
            parameters,
            strict: function.strict,
        };
        Ok(spec)
    }
}

impl From<FunctionSpec> for Function {
    fn from(tool: FunctionSpec) -> Self {
        Function {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters.to_value(),
            strict: tool.strict,
        }
    }
}

impl From<FunctionSpec> for ToolDefinition {
    fn from(tool: FunctionSpec) -> Self {
        ToolDefinition::Function(Function::from(tool))
    }
}

impl TryFrom<FunctionObject> for FunctionSpec {
    type Error = serde_json::Error;

    fn try_from(function: FunctionObject) -> Result<Self, Self::Error> {
        let parameters = match function.parameters {
            Some(params) => Schema::try_from(params)?,
            None => schema_for!(EmptyFunctionInput),
        };
        let spec = FunctionSpec {
            name: function.name,
            description: function.description,
            parameters,
            strict: function.strict.unwrap_or_default(),
        };
        Ok(spec)
    }
}

impl From<FunctionSpec> for FunctionObject {
    fn from(tool: FunctionSpec) -> Self {
        FunctionObject {
            name: tool.name,
            description: tool.description,
            parameters: Some(tool.parameters.to_value()),
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
