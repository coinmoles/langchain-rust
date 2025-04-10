use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use serde_json::Value;

use crate::{tools::Tool, utils::helper::normalize_tool_name};

#[async_trait]
pub trait Toolbox: Send + Sync {
    fn name(&self) -> String;

    async fn get_tools(
        &self,
    ) -> Result<&HashMap<String, Box<dyn Tool>>, Box<dyn Error + Send + Sync>>;

    async fn get_tool(&self, tool_name: &str) -> Result<&dyn Tool, Box<dyn Error + Send + Sync>> {
        let tool_name = normalize_tool_name(tool_name);
        let tools = self.get_tools().await?;

        tools
            .get(&tool_name)
            .map(|t| t.as_ref())
            .ok_or(format!("Tool {} not found", tool_name).into())
    }

    async fn call_tool(
        &self,
        tool_name: &str,
        input: Value,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let tool = self.get_tool(tool_name).await?;

        tool.call(input).await
    }
}
