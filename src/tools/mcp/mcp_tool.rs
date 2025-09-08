use std::{collections::HashMap, sync::Arc};

use futures::{stream, StreamExt, TryStreamExt};
use rmcp::{transport::StreamableHttpClientTransport, ServiceExt};
use secrecy::{ExposeSecret, SecretString};

use crate::{
    tools::{FunctionTool, McpError, McpFunctionTool, McpService},
    utils::helper::normalize_tool_name,
};

pub struct McpTool {
    uri: SecretString,
    name: String,
}

impl McpTool {
    pub fn new(uri: &str, name: impl Into<String>) -> Self {
        Self {
            uri: SecretString::from(uri),
            name: name.into(),
        }
    }

    pub async fn into_function_tools(
        predicates: &[Self],
    ) -> Result<HashMap<String, Box<dyn FunctionTool>>, McpError> {
        // Group tools by URI to minimize the number of connections
        let grouped = group_tools_by_uri(predicates);
        let merged: HashMap<String, Box<dyn FunctionTool>> = stream::iter(grouped.into_iter())
            .map(|(uri, preds)| fetch_tools(uri, preds))
            .buffer_unordered(8)
            .try_fold(HashMap::new(), |mut acc, map| async move {
                acc.extend(map);
                Ok(acc)
            })
            .await?;

        Ok(merged)
    }
}

/// Helper function to group mcp tools by their URI.
fn group_tools_by_uri(predicates: &[McpTool]) -> HashMap<&str, Vec<&str>> {
    let mut m: HashMap<&str, Vec<&str>> = HashMap::new();
    for p in predicates {
        let uri = p.uri.expose_secret();
        m.entry(uri).or_default().push(p.name.as_str());
    }
    m
}

async fn init_service(uri: &str) -> Result<McpService, McpError> {
    let transport = StreamableHttpClientTransport::from_uri(uri);
    let client_info = rmcp::model::ClientInfo::default();
    let service = client_info
        .serve(transport)
        .await
        .inspect_err(|e| tracing::error!("client error: {e:?}"))?;

    Ok(service)
}

async fn fetch_tools(
    uri: &str,
    names: Vec<&str>,
) -> Result<HashMap<String, Box<dyn FunctionTool>>, McpError> {
    let service = Arc::new(init_service(uri).await?);
    let mut tools = service
        .list_all_tools()
        .await?
        .into_iter()
        .map(|tool| -> Result<_, McpError> {
            let tool = McpFunctionTool::from_rmcp_tool(&service, tool)?;
            Ok((tool.name(), tool))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let mut out: HashMap<String, Box<dyn FunctionTool>> = HashMap::new();
    for name in names {
        let name = normalize_tool_name(name);
        let Some(tool) = tools.remove(&name) else {
            return Err(McpError::ToolNotFound(name));
        };
        out.insert(name, Box::new(tool));
    }

    Ok(out)
}
