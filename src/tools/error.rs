use thiserror::Error;

#[cfg(feature = "mcp")]
use crate::tools::McpError;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Error while running tool: {0}")]
    ExecutionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Input parsing error: {0}")]
    InputParseError(#[from] serde_json::Error),

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    McpError(Box<McpError>),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}

impl ToolError {
    pub fn execution_error<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        ToolError::ExecutionError(Box::new(error))
    }
}

impl From<McpError> for ToolError {
    fn from(error: McpError) -> Self {
        ToolError::McpError(Box::new(error))
    }
}
