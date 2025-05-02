use async_trait::async_trait;
use rmcp::model::CallToolRequestParam;
use serde_json::Value;
use url::Url;

use std::{borrow::Cow, collections::HashMap, error::Error, sync::Arc};

use crate::{
    tools::{tool_field::ToolParameters, ToolFunction},
    utils::helper::normalize_tool_name,
};

use super::{create_mcp_client, parse_mcp_response, McpService};

pub struct McpTool {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    parameters: ToolParameters,
    client: Arc<McpService>,
}

impl McpTool {
    pub fn new(
        name: Cow<'static, str>,
        description: Cow<'static, str>,
        parameters: ToolParameters,
        client: Arc<McpService>,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            client,
        }
    }

    pub async fn fetch_tools(
        url: Url,
        names: Option<impl IntoIterator<Item = impl AsRef<str>>>,
    ) -> Result<HashMap<String, Self>, Box<dyn Error + Send + Sync>> {
        let client = create_mcp_client(url).await?;
        let client = Arc::new(client);

        let tools = client
            .list_all_tools()
            .await?
            .into_iter()
            .map(|tool| -> Result<(String, McpTool), serde_json::Error> {
                let tool_name = normalize_tool_name(tool.name.as_ref());
                let mcp_tool = McpTool::new(
                    tool.name,
                    tool.description,
                    tool.input_schema.as_ref().try_into()?,
                    Arc::clone(&client),
                );

                Ok((tool_name, mcp_tool))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let Some(names) = names else {
            return Ok(tools);
        };

        let mut tools = tools;
        let filtered_tools = names
            .into_iter()
            .filter_map(|tool_name| {
                let tool_name = normalize_tool_name(tool_name.as_ref());
                match tools.remove(&tool_name) {
                    Some(tool) => Some((tool_name, tool)),
                    None => {
                        log::warn!("Tool {} not found", tool_name);
                        None
                    }
                }
            })
            .collect::<HashMap<_, _>>();
        Ok(filtered_tools)
    }

    pub async fn fetch_tool(
        url: Url,
        name: impl AsRef<str>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let tool_name = normalize_tool_name(name.as_ref());
        Self::fetch_tools(url, Some([&tool_name]))
            .await?
            .remove(&tool_name)
            .ok_or(format!("Tool {} not found", tool_name).into())
    }
}

#[async_trait]
impl ToolFunction for McpTool {
    type Input = Value;
    type Result = String;

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn description(&self) -> String {
        self.description.to_string()
    }

    fn parameters(&self) -> ToolParameters {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        false
    }

    async fn run(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let input = match input {
            Value::Object(obj) => obj,
            _ => {
                return Err("Invalid input".into());
            }
        };

        let tool_result: rmcp::model::CallToolResult = self
            .client
            .call_tool(CallToolRequestParam {
                name: self.name.clone().into(),
                arguments: Some(input),
            })
            .await?;

        let content = tool_result
            .content
            .into_iter()
            .map(parse_mcp_response)
            .collect::<Vec<_>>()
            .join("\n");

        if tool_result.is_error.unwrap_or(false) {
            Err(content.into())
        } else {
            Ok(content.into())
        }
    }
}
