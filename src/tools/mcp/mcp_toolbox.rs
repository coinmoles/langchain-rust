use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::SseTransport,
    ServiceExt,
};

use std::{error::Error, sync::Arc};

use crate::tools::Tool;

use super::{McpService, McpTool};

pub struct McpToolbox {
    client: Arc<McpService>,
    tools: Vec<Box<dyn Tool>>,
}

impl McpToolbox {
    pub async fn build(url: String) -> Result<Self, Box<dyn Error + Send + Sync>> {
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

                Ok(Box::new(tool) as Box<dyn Tool>)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(McpToolbox { client: Arc::clone(&client), tools })
    }

    pub async fn get_tools(&self) -> &Vec<Box<dyn Tool>> {
        &self.tools
    }
}
