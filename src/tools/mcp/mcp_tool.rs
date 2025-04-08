use async_trait::async_trait;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::SseTransport,
    ServiceExt,
};
use serde_json::Value;
use url::Url;

use std::{borrow::Cow, error::Error, sync::Arc};

use crate::tools::{tool_field::ToolParameters, Tool};

use super::{parse_mcp_response, McpService};

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
}

impl McpTool {
    pub async fn get_tools_from_server(
        url: Url,
    ) -> Result<Vec<Box<dyn Tool>>, Box<dyn Error + Send + Sync>> {
        let transport = SseTransport::start(url).await?;

        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "MCP Client".to_string(),
                version: "0.0.1".to_string(),
            },
        };

        let client = client_info
            .serve(transport)
            .await
            .inspect_err(|e| log::error!("Failed to connect to MCP server: {:?}", e))?;

        let client = Arc::new(client);

        let tools = client
            .list_all_tools()
            .await?
            .into_iter()
            .map(|tool| -> Result<Box<dyn Tool>, serde_json::Error> {
                let tool = McpTool::new(
                    tool.name.into(),
                    tool.description.into(),
                    (*tool.input_schema).clone().try_into()?,
                    Arc::clone(&client),
                );

                Ok(Box::new(tool))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tools)
    }
}

#[async_trait]
impl Tool for McpTool {
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
        true
    }

    async fn call(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
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
