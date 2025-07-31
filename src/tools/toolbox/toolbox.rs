use std::collections::HashMap;

use async_trait::async_trait;
use serde_json::Value;

use crate::{
    tools::{ToolDyn, ToolError, ToolOutput},
    utils::helper::normalize_tool_name,
};

#[async_trait]
pub trait Toolbox: Send + Sync {
    fn name(&self) -> String;

    fn get_tools(&self) -> HashMap<&str, &dyn ToolDyn>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolDyn> {
        let tool_name = normalize_tool_name(tool_name);
        let tools = self.get_tools();

        tools.get(tool_name.as_str()).copied()
    }

    async fn call_tool(&self, tool_name: &str, input: Value) -> Result<ToolOutput, ToolError> {
        let tool = self
            .get_tool(tool_name)
            .ok_or(ToolError::ToolNotFound(tool_name.into()))?;

        tool.call(input).await
    }
}
