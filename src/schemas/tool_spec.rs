use async_openai::types::responses::{Function, ToolDefinition};
use async_openai::types::{ChatCompletionTool, FunctionObject};
use indoc::formatdoc;
use schemars::{Schema, schema_for};
use serde_json::json;

use crate::tools::{EmptyFunctionInput, FunctionTool, McpTool, describe_parameters};
use crate::utils::helper::normalize_tool_name;

/// The specification of tools available to the LLM.
///
/// # Fields
/// - `functions`: A list of function tools.
/// - `mcps`: A list of MCP tools.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// A list of functions available to the LLM.
    pub functions: Vec<FunctionSpec>,
    /// A list of MCP tools available to the LLM.
    pub mcps: Vec<McpTool>,
}

impl ToolSpec {
    /// Constructs a new `ToolSpec`.
    pub fn new(functions: Vec<FunctionSpec>, mcps: Vec<McpTool>) -> Option<Self> {
        if functions.is_empty() && mcps.is_empty() {
            return None;
        }
        Some(Self { functions, mcps })
    }

    /// Constructs a new `ToolSpec` from a list of function tools and MCP tools.
    pub fn from_tools(functions: &[&dyn FunctionTool], mcps: Vec<McpTool>) -> Option<Self> {
        let functions = functions.iter().map(|f| f.get_spec()).collect();
        Self::new(functions, mcps)
    }

    /// Returns true if there are no tools defined.
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.mcps.is_empty()
    }

    /// Converts the `ToolSpec` into a list of `ToolDefinition`s.
    pub fn into_tool_definitions(self) -> Vec<ToolDefinition> {
        let mut definitions: Vec<ToolDefinition> =
            self.functions.into_iter().map(Into::into).collect();
        definitions.extend(
            McpTool::into_definitions(self.mcps)
                .into_iter()
                .map(ToolDefinition::Mcp),
        );
        definitions
    }
}

/// The specification of a function tool available to the LLM.
///
/// Corresponds to [`FunctionObject`](async_openai::types::FunctionObject) for the chat completions
/// api and [`Function`](async_openai::types::responses::Function) for the responses api.
///
/// # Fields
/// - `name`: The name of the function.
/// - `description`: A description of the function.
/// - `parameters`: The parameters of the function as a JSON schema.
/// - `strict`: Whether the function should be called with strict parameter validation.
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    /// The name of the function.
    pub name: String,
    /// A description of the function.
    pub description: Option<String>,
    /// The parameters of the function as a JSON schema.
    pub parameters: Schema,
    /// Whether the function should be called with strict parameter validation.
    pub strict: bool,
}

impl FunctionSpec {
    /// Constructs a new `FunctionSpec`.
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

    /// Returns the JSON representation of the function specification.
    pub fn as_json(&self) -> String {
        let json = json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters,
            "strict": self.strict,
        });
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
    }

    /// Returns a human-readable description of the function specification.
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
