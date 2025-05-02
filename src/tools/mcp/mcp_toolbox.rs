use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use tokio::sync::OnceCell;
use url::Url;

use crate::tools::{Tool, Toolbox};

use super::McpTool;

pub struct McpToolbox {
    pub name: String,
    pub url: Url,
    pub using: Option<Vec<String>>,
    pub tools: OnceCell<HashMap<String, McpTool>>,
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
}

#[async_trait]
impl Toolbox for McpToolbox {
    fn name(&self) -> String {
        self.name.clone()
    }

    async fn get_tools(&self) -> Result<HashMap<&str, &dyn Tool>, Box<dyn Error + Send + Sync>> {
        let tools = self
            .tools
            .get_or_try_init(|| McpTool::fetch_tools(self.url.clone(), self.using.clone()))
            .await?
            .iter()
            .map(|(k, v)| (k.as_str(), v as &dyn Tool))
            .collect();

        Ok(tools)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

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

        let tools = toolbox.get_tools().await.unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.parameters().to_openai_field())
        }
    }

    #[tokio::test]
    async fn test_mcp_toolbox_using() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let toolbox = McpToolbox::new("Test", url, Some(vec!["say_hello".into(), "sum".into()]));

        let tools = toolbox.get_tools().await.unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.into_openai_tool());
            println!("{:#?}", tool.usage_limit());
        }
    }
}
