use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
    tools::{Tool, ToolError},
    utils::helper::normalize_tool_name,
};

#[async_trait]
pub trait Toolbox: Send + Sync {
    fn name(&self) -> String;

    fn get_tools(&self) -> Result<HashMap<&str, &dyn Tool>, ToolError>;

    fn get_tool(&self, tool_name: &str) -> Result<&dyn Tool, ToolError> {
        let tool_name = normalize_tool_name(tool_name);
        let tools = self.get_tools()?;

        tools
            .get(tool_name.as_str())
            .copied()
            .ok_or(ToolError::ToolNotFound(tool_name))
    }

    async fn call_tool(&self, tool_name: &str, input: Value) -> Result<String, ToolError> {
        let tool = self.get_tool(tool_name)?;

        tool.call(input).await
    }
}
