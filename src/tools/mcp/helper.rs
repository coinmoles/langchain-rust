use std::{collections::HashMap, sync::Arc};

use rmcp::{
    model::{Annotated, RawContent, ResourceContents},
    transport::StreamableHttpClientTransport,
    ServiceExt,
};

use crate::{
    tools::{FunctionTool, McpError, McpFunctionTool, McpService},
    utils::helper::normalize_tool_name,
};

async fn init_service(uri: &str) -> Result<McpService, McpError> {
    let transport = StreamableHttpClientTransport::from_uri(uri);
    let client_info = rmcp::model::ClientInfo::default();
    let service = client_info
        .serve(transport)
        .await
        .inspect_err(|e| tracing::error!("client error: {e:?}"))?;

    Ok(service)
}

pub(super) async fn fetch_tools(
    uri: String,
    names: Option<Vec<String>>,
) -> Result<HashMap<String, Box<dyn FunctionTool>>, McpError> {
    let service = Arc::new(init_service(&uri).await?);
    let mut map = HashMap::new();
    for tool in service.list_all_tools().await? {
        let tool = McpFunctionTool::from_rmcp_tool(&service, tool)?;
        map.insert(tool.name(), tool);
    }

    let Some(names) = names else {
        return Ok(map
            .into_iter()
            .map(|(k, v)| (k, Box::new(v) as Box<dyn FunctionTool>))
            .collect());
    };
    let mut out: HashMap<String, Box<dyn FunctionTool>> = HashMap::new();
    for name in names {
        let name = normalize_tool_name(&name);
        let Some(tool) = map.remove(&name) else {
            return Err(McpError::ToolNotFound(name));
        };
        out.insert(name, Box::new(tool));
    }

    Ok(out)
}

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
                ..
            } => {
                format!(
                    "[Resource]({uri}){}: {text}",
                    mime_type.map(|s| format!(" ({s})")).unwrap_or_default(),
                )
            }
            ResourceContents::BlobResourceContents {
                uri,
                mime_type,
                blob,
                ..
            } => {
                format!(
                    "[Resource]({uri}){}: {blob}",
                    mime_type.map(|s| format!(" ({s})")).unwrap_or_default(),
                )
            }
        },
        _ => {
            tracing::warn!("Unsupported MCP response type: {:?}", response.raw);
            "Unsupported response type".to_string()
        }
    }
}
