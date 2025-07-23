use async_trait::async_trait;
use rmcp::model::CallToolRequestParam;
use schemars::schema::RootSchema;
use serde_json::Value;

use std::{borrow::Cow, error::Error, sync::Arc};

use crate::tools::{Tool, ToolError};

use super::{parse_mcp_response, McpService, McpServiceExt};

pub struct McpTool {
    client: Arc<McpService>,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    parameters: RootSchema,
}

impl McpTool {
    pub fn new(
        client: Arc<McpService>,
        name: Cow<'static, str>,
        description: Cow<'static, str>,
        parameters: RootSchema,
    ) -> Self {
        Self {
            client,
            name,
            description,
            parameters,
        }
    }

    pub async fn fetch_tool(
        client: impl Into<Arc<McpService>>,
        name: impl AsRef<str> + Send + Sync,
    ) -> Result<Self, ToolError> {
        let client: Arc<McpService> = client.into();
        client.fetch_tool(name.as_ref()).await
    }
}

#[async_trait]
impl Tool for McpTool {
    type Input = Value;
    type Output = Vec<String>;

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn description(&self) -> String {
        self.description.to_string()
    }

    fn parameters(&self) -> RootSchema {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        false
    }

    async fn run(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let input = match input {
            Value::Object(obj) => obj,
            _ => {
                return Err("Invalid input".into());
            }
        };

        let tool_result: rmcp::model::CallToolResult = self
            .client
            .call_tool(CallToolRequestParam {
                name: self.name.clone(),
                arguments: Some(input),
            })
            .await?;

        let content = tool_result
            .content
            .into_iter()
            .map(parse_mcp_response)
            .collect::<Vec<_>>();

        if tool_result.is_error.unwrap_or(false) {
            Err(content.join("\n").into())
        } else {
            Ok(content)
        }
    }
}
