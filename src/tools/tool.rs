use crate::{
    schemas::ToolSpec,
    tools::{FunctionTool, McpFunctionTool, ToolError, ToolOutput},
};

pub enum Tool {
    Function(Box<dyn FunctionTool>),
    Mcp(Box<McpFunctionTool>),
}

impl Tool {
    pub async fn get_spec(&self) -> ToolSpec {
        match self {
            Tool::Function(func) => func.get_spec(),
            Tool::Mcp(mcp) => mcp.get_spec(),
        }
    }

    pub async fn call(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        match self {
            Tool::Function(func) => func.call(input).await,
            Tool::Mcp(mcp) => mcp.call(input).await,
        }
    }
}
