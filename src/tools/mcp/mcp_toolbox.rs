use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use rmcp::{
    model::{ClientCapabilities, ClientInfo, Implementation},
    transport::SseTransport,
    ServiceExt,
};
use tokio::sync::OnceCell;
use url::Url;

use crate::{
    tools::{Tool, Toolbox},
    utils::helper::normalize_tool_name,
};

use super::McpTool;

pub struct McpToolbox {
    pub name: String,
    pub url: Url,
    pub using: Option<Vec<String>>,
    pub tools: OnceCell<HashMap<String, Box<dyn Tool>>>,
}

impl McpToolbox {
    pub fn new<S: Into<String>>(name: S, url: Url, using: Option<Vec<String>>) -> Self {
        Self {
            name: name.into(),
            url,
            using,
            tools: OnceCell::new(),
        }
    }

    pub async fn get_tools(
        &self,
    ) -> Result<&HashMap<String, Box<dyn Tool>>, Box<dyn Error + Send + Sync>> {
        self.tools
            .get_or_try_init(|| async {
                // TODO: Support other transport types
                let transport = SseTransport::start(self.url.clone()).await?;

                // TODO: Support other client info
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
                    .map(
                        |tool| -> Result<(String, Box<dyn Tool>), serde_json::Error> {
                            let tool_name = normalize_tool_name(tool.name.as_ref());

                            let mcp_tool = McpTool::new(
                                tool.name.into(),
                                tool.description.into(),
                                tool.input_schema.as_ref().try_into()?,
                                Arc::clone(&client),
                            );

                            Ok((tool_name, Box::new(mcp_tool)))
                        },
                    )
                    .collect::<Result<HashMap<_, _>, _>>()?;

                if let Some(using) = &self.using {
                    let mut tools = tools;
                    let filtered_tools = using
                        .iter()
                        .filter_map(|tool_name| {
                            let tool_name = normalize_tool_name(tool_name);
                            match tools.remove(&tool_name) {
                                Some(tool) => Some((tool_name, tool)),
                                None => {
                                    log::warn!(
                                        "Tool {} not found in toolbox {}",
                                        tool_name,
                                        self.name
                                    );
                                    None
                                }
                            }
                        })
                        .collect::<HashMap<_, _>>();
                    Ok(filtered_tools)
                } else {
                    Ok(tools)
                }
            })
            .await
    }
}

#[async_trait]
impl Toolbox for McpToolbox {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn get_tools(
        &self,
    ) -> Result<&HashMap<String, Box<dyn Tool>>, Box<dyn Error + Send + Sync>> {
        self.get_tools().await
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::tools::ListTools;

    use super::*;

    #[tokio::test]
    async fn test_list_tools() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let toolbox = McpToolbox::new("Test", url, None);

        let list_tools_tool = ListTools::new(&Arc::new(toolbox));
        println!("{:#?}", list_tools_tool.into_openai_tool());
        println!("{}", list_tools_tool.call(json!({})).await.unwrap());
    }

    #[tokio::test]
    async fn test_mcp_toolbox() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let toolbox = McpToolbox::new("Test", url, None);

        let tools = toolbox
            .get_tools()
            .await
            .unwrap()
            .values()
            .collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.parameters().to_openai_field())
        }
    }

    #[tokio::test]
    async fn test_mcp_toolbox_using() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let toolbox = McpToolbox::new("Test", url, Some(vec!["say_hello".into(), "sum".into()]));

        let tools = toolbox
            .get_tools()
            .await
            .unwrap()
            .values()
            .collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.into_openai_tool());
            println!("{:#?}", tool.usage_limit());
        }
    }
}
