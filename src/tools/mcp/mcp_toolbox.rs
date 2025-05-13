use std::{borrow::Cow, collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;

use crate::tools::{Tool, Toolbox};

use super::{McpService, McpServiceExt, McpTool};

pub struct McpToolbox {
    pub client: Arc<McpService>,
    pub name: Cow<'static, str>,
    pub tools: HashMap<String, McpTool>,
}

impl McpToolbox {
    pub fn new(
        client: impl Into<Arc<McpService>>,
        name: impl Into<Cow<'static, str>>,
        tools: HashMap<String, McpTool>,
    ) -> Self {
        Self {
            client: client.into(),
            name: name.into(),
            tools,
        }
    }

    pub async fn fetch(
        client: impl Into<Arc<McpService>>,
        name: impl Into<Cow<'static, str>>,
        using: Option<Vec<Cow<'static, str>>>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = client.into();
        let tools = client.fetch_tools(using.clone()).await?;

        Ok(Self::new(client, name, tools))
    }
}

#[async_trait]
impl Toolbox for McpToolbox {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn get_tools(&self) -> Result<HashMap<&str, &dyn Tool>, Box<dyn Error + Send + Sync>> {
        let tools = self
            .tools
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
    use url::Url;

    use crate::tools::{ListTools, McpServiceFromUrl};

    use super::*;

    #[tokio::test]
    async fn test_list_tools() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url).await.unwrap();
        let toolbox = McpToolbox::fetch(client, "Test", None).await.unwrap();

        let list_tools_tool = ListTools::new(&Arc::new(toolbox));
        println!("{:#?}", list_tools_tool.as_openai_tool());
        println!("{}", list_tools_tool.call(json!({})).await.unwrap());
    }

    #[tokio::test]
    async fn test_mcp_toolbox() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url.clone()).await.unwrap();
        let toolbox = McpToolbox::fetch(client, "Test", None).await.unwrap();

        let tools = toolbox.get_tools().unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.parameters().to_openai_field())
        }
    }

    #[tokio::test]
    async fn test_mcp_toolbox_using() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url.clone()).await.unwrap();
        let toolbox =
            McpToolbox::fetch(client, "Test", Some(vec!["say_hello".into(), "sum".into()]))
                .await
                .unwrap();

        let tools = toolbox.get_tools().unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.as_openai_tool());
            println!("{:#?}", tool.usage_limit());
        }
    }
}
