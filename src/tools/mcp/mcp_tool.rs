use std::collections::HashMap;

use futures::{stream, StreamExt, TryStreamExt};
use secrecy::{ExposeSecret, SecretString};

use crate::tools::{mcp::fetch_tools, FunctionTool, McpError};

#[derive(Debug, Clone)]
pub struct McpTool {
    uri: SecretString,
    pub name: String,
}

impl McpTool {
    pub fn new(uri: &str, name: impl Into<String>) -> Self {
        Self {
            uri: SecretString::from(uri),
            name: name.into(),
        }
    }

    pub async fn into_function_tool(self) -> Result<Box<dyn FunctionTool>, McpError> {
        let tool = McpTool::into_function_tools(vec![self])
            .await?
            .into_values()
            .next()
            .expect("One tool should be present");
        Ok(tool)
    }

    pub async fn into_function_tools(
        predicates: Vec<Self>,
    ) -> Result<HashMap<String, Box<dyn FunctionTool>>, McpError> {
        // Group tools by URI to minimize the number of connections
        let grouped = group_tools_by_uri(predicates);
        let merged = stream::iter(grouped.into_iter())
            .map(|(uri, names)| fetch_tools(uri, Some(names)))
            .buffer_unordered(8)
            .try_concat()
            .await?;

        Ok(merged)
    }
}

/// Helper function to group mcp tools by their URI.
fn group_tools_by_uri(predicates: Vec<McpTool>) -> HashMap<String, Vec<String>> {
    let mut m: HashMap<String, Vec<String>> = HashMap::new();
    for p in predicates {
        let uri = p.uri.expose_secret().to_string();
        m.entry(uri).or_default().push(p.name);
    }
    m
}
