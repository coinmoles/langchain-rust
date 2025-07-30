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
}
