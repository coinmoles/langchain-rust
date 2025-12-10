use std::collections::HashMap;

use async_trait::async_trait;

use crate::tools::mcp::fetch_tools;
use crate::tools::{FunctionTool, McpError, SimpleToolbox, Toolbox};

pub struct McpToolbox(SimpleToolbox);

impl McpToolbox {
    pub async fn fetch(
        name: impl Into<String>,
        url: impl Into<String>,
        using: Option<Vec<String>>,
    ) -> Result<Self, McpError> {
        let tools = fetch_tools(url.into(), using).await?;
        let toolbox = SimpleToolbox::new(name, tools);
        Ok(Self(toolbox))
    }
}

#[async_trait]
impl Toolbox for McpToolbox {
    fn name(&self) -> String {
        self.0.name()
    }

    fn get_tools(&self) -> HashMap<&str, &dyn FunctionTool> {
        self.0.get_tools()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use super::*;
    use crate::tools::ListTools;

    #[tokio::test]
    #[ignore = "Requires running mcp server"]
    async fn test_list_tools() {
        let url = "http://localhost:8000/sse";
        let toolbox = McpToolbox::fetch("Test", url, None).await.unwrap();

        let list_tools_tool = ListTools::new(&Arc::new(toolbox));
        println!("{:#?}", list_tools_tool.get_spec());
        println!("{}", list_tools_tool.call(json!({})).await.unwrap().data);
    }

    #[tokio::test]
    #[ignore = "Requires running mcp server"]
    async fn test_mcp_toolbox() {
        let url = "http://localhost:8000/sse";
        let toolbox = McpToolbox::fetch("Test", url, None).await.unwrap();

        let tools = toolbox.get_tools();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.parameters())
        }
    }

    #[tokio::test]
    #[ignore = "Requires running mcp server"]
    async fn test_mcp_toolbox_using() {
        let url = "http://localhost:8000/sse";
        let tools = vec!["say_hello".into(), "sum".into()];
        let toolbox = McpToolbox::fetch("Test", url, Some(tools)).await.unwrap();

        let tools = toolbox.get_tools();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.get_spec());
            println!("{:#?}", tool.usage_limit());
        }
    }
}
