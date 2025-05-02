use std::error::Error;

use reqwest::IntoUrl;
use rmcp::{
    model::{Annotated, RawContent, ResourceContents},
    transport::SseTransport,
    ServiceExt,
};

pub(super) fn parse_mcp_response(response: Annotated<RawContent>) -> String {
    match response.raw {
        RawContent::Text(content) => content.text,
        RawContent::Image(content) => content.data,
        // TODO: improve resource content parsing
        RawContent::Resource(content) => match content.resource {
            ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
            } => {
                format!(
                    "[Resource]({}){}: {}",
                    uri,
                    mime_type.map(|s| format!(" ({})", s)).unwrap_or_default(),
                    text,
                )
            }
            ResourceContents::BlobResourceContents {
                uri,
                mime_type,
                blob,
            } => {
                format!(
                    "[Resource]({}){}: {}",
                    uri,
                    mime_type.map(|s| format!(" ({})", s)).unwrap_or_default(),
                    blob
                )
            }
        },
    }
}

type McpClient =
    rmcp::service::RunningService<rmcp::RoleClient, rmcp::model::InitializeRequestParam>;

pub(super) async fn create_mcp_client(
    url: impl IntoUrl,
) -> Result<McpClient, Box<dyn Error + Send + Sync>> {
    let transport = SseTransport::start(url).await?;

    let client_info = rmcp::model::ClientInfo {
        protocol_version: Default::default(),
        capabilities: rmcp::model::ClientCapabilities::default(),
        client_info: rmcp::model::Implementation {
            name: "MCP Client".to_string(),
            version: "0.0.1".to_string(),
        },
    };

    let client = client_info
        .serve(transport)
        .await
        .map_err(|e| format!("Failed to connect to MCP server: {:?}", e))?;

    Ok(client)
}
