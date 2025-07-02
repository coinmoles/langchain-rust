use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use reqwest::IntoUrl;
use rmcp::{
    model::InitializeRequestParam,
    service::RunningService,
    transport::{sse::SseTransportError, SseTransport},
    RoleClient, ServiceExt,
};

use crate::{tools::ToolError, utils::helper::normalize_tool_name};

use super::McpTool;

pub type McpService = RunningService<RoleClient, InitializeRequestParam>;

#[async_trait]
pub trait McpServiceFromUrl: Send + Sync {
    async fn from_url(url: impl IntoUrl + Send + Sync) -> Result<Self, SseTransportError>
    where
        Self: Sized;
}

#[async_trait]
impl McpServiceFromUrl for McpService {
    async fn from_url(url: impl IntoUrl + Send + Sync) -> Result<Self, SseTransportError> {
        let transport = SseTransport::start(url).await?;

        let client_info = rmcp::model::ClientInfo {
            protocol_version: Default::default(),
            capabilities: Default::default(),
            client_info: rmcp::model::Implementation {
                name: "MCP Client".to_string(),
                version: "0.0.1".to_string(),
            },
        };

        let client = client_info.serve(transport).await?;

        Ok(client)
    }
}

#[async_trait]
pub trait McpServiceExt: Send + Sync {
    async fn fetch_tools(
        &self,
        names: Option<impl IntoIterator<Item = impl AsRef<str>> + Send + Sync>,
    ) -> Result<HashMap<String, McpTool>, ToolError>;

    async fn fetch_tool(&self, name: impl AsRef<str> + Send + Sync) -> Result<McpTool, ToolError> {
        let tool_name = normalize_tool_name(name.as_ref());
        self.fetch_tools(Some([&tool_name]))
            .await?
            .remove(&tool_name)
            .ok_or(ToolError::ToolNotFound(tool_name))
    }
}

#[async_trait]
impl McpServiceExt for Arc<McpService> {
    async fn fetch_tools(
        &self,
        names: Option<impl IntoIterator<Item = impl AsRef<str>> + Send + Sync>,
    ) -> Result<HashMap<String, McpTool>, ToolError> {
        let tools = self
            .list_all_tools()
            .await?
            .into_iter()
            .map(|tool| -> Result<(String, McpTool), serde_json::Error> {
                let tool_name = normalize_tool_name(tool.name.as_ref());
                let parameters = tool.schema_as_json_value();
                let mcp_tool = McpTool::new(
                    Arc::clone(self),
                    tool.name,
                    tool.description,
                    serde_json::from_value(parameters)?,
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
                        log::warn!("Tool {tool_name} not found");
                        None
                    }
                }
            })
            .collect::<HashMap<_, _>>();
        Ok(filtered_tools)
    }
}
