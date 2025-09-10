use async_trait::async_trait;
use rmcp::model::CallToolRequestParam;
use schemars::Schema;
use serde::de::Error;
use serde_json::Value;

use std::sync::Arc;

use crate::{
    tools::{sealed, FunctionTool, McpError, ToolError, ToolOutput},
    utils::helper::normalize_tool_name,
};

use super::{parse_mcp_response, McpService};

/// A function tool representation of an MCP tool.
///
/// This is used for models that do not support native MCP tool calling.
/// For such models, this struct is used to treat MCP tools as standard function tools.
pub struct McpFunctionTool {
    client: Arc<McpService>,
    name: String,
    description: Option<String>,
    parameters: Schema,
}

impl McpFunctionTool {
    pub fn new(
        client: Arc<McpService>,
        name: String,
        description: Option<String>,
        parameters: Schema,
    ) -> Self {
        Self {
            client,
            name,
            description,
            parameters,
        }
    }

    // TODO: handle RootSchema better
    pub fn from_rmcp_tool(
        service: &Arc<McpService>,
        tool: rmcp::model::Tool,
    ) -> Result<Self, serde_json::Error> {
        let parameters = tool.schema_as_json_value();
        let name = normalize_tool_name(tool.name.as_ref());
        let description = tool.description.map(|d| d.into_owned()); // TODO: do something about RootSchema
        let tool = McpFunctionTool::new(
            Arc::clone(service),
            name.clone(),
            description,
            Schema::try_from(parameters)?,
        );
        Ok(tool)
    }
}

impl sealed::Sealed for McpFunctionTool {}

#[async_trait]
impl FunctionTool for McpFunctionTool {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn description(&self) -> String {
        self.description
            .as_ref()
            .map_or_else(|| "No description provided".to_string(), |d| d.to_string())
    }

    fn parameters(&self) -> Schema {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        false
    }

    async fn call(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let input = match input {
            Value::Object(obj) => obj,
            _ => {
                return Err(ToolError::InputParseError(serde_json::Error::custom(
                    "Expected a JSON object as input",
                )))
            }
        };

        let tool_result: rmcp::model::CallToolResult = self
            .client
            .call_tool(CallToolRequestParam {
                name: self.name.clone().into(),
                arguments: Some(input),
            })
            .await
            .map_err(McpError::from)?;

        let content = tool_result
            .content
            .into_iter()
            .map(parse_mcp_response)
            .collect::<Vec<_>>();

        if tool_result.is_error.unwrap_or(false) {
            return Err(ToolError::ExecutionError(content.join("\n").into()));
        }
        Ok(ToolOutput {
            data: content.into(),
            summary: None,
        })
    }
}
