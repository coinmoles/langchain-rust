use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A document with content, metadata, and a score.
///
/// # Fields
/// - `content`: The main content of the document.
/// - `metadata`: A map of metadata properties associated with the document.
/// - `score`: A relevance score for the document.
///
/// # Usage
/// ```
/// use std::collections::HashMap;
///
/// use langchain_rust::schemas::Document;
/// use serde_json::json;
///
/// let my_doc = Document::new("This is the document content.".to_string())
///     .with_metadata(HashMap::from_iter([(
///         "author".to_string(),
///         json!("John Doe"),
///     )]))
///     .with_score(0.75);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The main content of the document.
    pub content: String,
    /// A map of metadata properties associated with the document.
    pub metadata: HashMap<String, Value>,
    /// A relevance score for the document.
    pub score: f64,
}

impl Document {
    /// Constructs a new `Document` with the given content.
    pub fn new(page_content: impl Into<String>) -> Self {
        Document {
            content: page_content.into(),
            metadata: HashMap::new(),
            score: 0.0,
        }
    }

    /// Sets the `metadata` of the document.
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the `score` of the document.
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }
}
