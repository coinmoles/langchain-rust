use async_trait::async_trait;
use rmcp::model::{CallToolRequestParam, RawContent, ResourceContents};
use serde_json::Value;

use std::{borrow::Cow, error::Error, sync::Arc};

use crate::tools::{tool_field::ToolParameters, Tool};

use super::McpService;

pub struct McpTool {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    parameters: ToolParameters,
    client: Arc<McpService>,
}

impl McpTool {
    pub fn new(
        name: Cow<'static, str>,
        description: Cow<'static, str>,
        parameters: ToolParameters,
        client: Arc<McpService>,
    ) -> Self {
        Self {
            name,
            description,
            parameters,
            client,
        }
    }
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> String {
        self.name.to_string()
    }

    fn description(&self) -> String {
        self.description.to_string()
    }

    fn parameters(&self) -> ToolParameters {
        self.parameters.clone()
    }

    fn strict(&self) -> bool {
        true
    }

    async fn call(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let input = match input {
            Value::Object(obj) => obj,
            _ => {
                return Err("Invalid input".into());
            }
        };

        let tool_result: rmcp::model::CallToolResult = self
            .client
            .call_tool(CallToolRequestParam {
                name: self.name.clone().into(),
                arguments: Some(input),
            })
            .await?;

        let content = tool_result
            .content
            .into_iter()
            .map(|v: rmcp::model::Annotated<RawContent>| match v.raw {
                RawContent::Text(content) => content.text,
                RawContent::Image(content) => content.data,
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
            })
            .collect::<Vec<_>>()
            .join("\n");

        if tool_result.is_error.unwrap_or(false) {
            Err(content.into())
        } else {
            Ok(content.into())
        }
    }
}
