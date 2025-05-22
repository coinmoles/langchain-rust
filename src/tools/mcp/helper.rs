use rmcp::model::{Annotated, RawContent, ResourceContents};

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
                    "[Resource]({uri}){}: {text}",
                    mime_type.map(|s| format!(" ({s})")).unwrap_or_default(),
                )
            }
            ResourceContents::BlobResourceContents {
                uri,
                mime_type,
                blob,
            } => {
                format!(
                    "[Resource]({uri}){}: {blob}",
                    mime_type.map(|s| format!(" ({s})")).unwrap_or_default(),
                )
            }
        },
    }
}
