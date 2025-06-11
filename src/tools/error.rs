use thiserror::Error;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Error while running tool: {0}")]
    ExecutionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Input parsing error: {0}")]
    InputParseError(#[from] serde_json::Error),

    #[cfg(feature = "mcp")]
    #[error("MCP sse transport error: {0}")]
    McpSseTransportError(#[from] rmcp::transport::sse::SseTransportError),

    #[cfg(feature = "mcp")]
    #[error("MCP error: {0}")]
    McpError(#[from] rmcp::ServiceError),

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
