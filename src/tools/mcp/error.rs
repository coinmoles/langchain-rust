use rmcp::service::ClientInitializeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("Service error: {0}")]
    ServiceError(rmcp::ServiceError),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Client initialize error: {0}")]
    ClientInitializeError(#[from] ClientInitializeError),
    #[error("Parameter specification deserialization error: {0}")]
    ParaSpecDeserializeError(#[from] serde_json::Error),
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}
