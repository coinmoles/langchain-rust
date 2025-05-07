use std::{borrow::Cow, collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;
use tokio::sync::OnceCell;

use crate::tools::{Tool, Toolbox};

use super::{McpService, McpServiceExt, McpTool};

pub struct McpToolbox {
    pub client: Arc<McpService>,
    pub name: Cow<'static, str>,
    pub using: Option<Vec<Cow<'static, str>>>,
    pub tools: OnceCell<HashMap<String, McpTool>>,
}

impl McpToolbox {
    pub fn new(
        client: impl Into<Arc<McpService>>,
        name: impl Into<Cow<'static, str>>,
        using: Option<Vec<Cow<'static, str>>>,
    ) -> Self {
        Self {
            client: client.into(),
            name: name.into(),
            using,
            tools: OnceCell::new(),
        }
    }
}

#[async_trait]
impl Toolbox for McpToolbox {
    fn name(&self) -> String {
        self.name.to_string()
    }

    async fn get_tools(&self) -> Result<HashMap<&str, &dyn Tool>, Box<dyn Error + Send + Sync>> {
        let tools = self
            .tools
            .get_or_try_init(|| self.client.fetch_tools(self.using.clone()))
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
    use url::Url;

    use crate::tools::{ListTools, McpServiceFromUrl};

    use super::*;

    #[tokio::test]
    async fn test_list_tools() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url).await.unwrap();
        let toolbox = McpToolbox::new(client, "Test", None);

        let list_tools_tool = ListTools::new(&Arc::new(toolbox));
        println!("{:#?}", list_tools_tool.into_openai_tool());
        println!("{}", list_tools_tool.call(json!({})).await.unwrap());
    }

    #[tokio::test]
    async fn test_mcp_toolbox() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url.clone()).await.unwrap();
        let toolbox = McpToolbox::new(client, "Test", None);

        let tools = toolbox.get_tools().await.unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.parameters().to_openai_field())
        }
    }

    #[tokio::test]
    async fn test_mcp_toolbox_using() {
        let url = Url::parse("http://localhost:8000/sse").unwrap();
        let client = McpService::from_url(url.clone()).await.unwrap();
        let toolbox = McpToolbox::new(client, "Test", Some(vec!["say_hello".into(), "sum".into()]));

        let tools = toolbox.get_tools().await.unwrap();
        let tools = tools.values().collect::<Vec<_>>();

        for tool in tools {
            println!("{:#?}", tool.into_openai_tool());
            println!("{:#?}", tool.usage_limit());
        }
    }
}
