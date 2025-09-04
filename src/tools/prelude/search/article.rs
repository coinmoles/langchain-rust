use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::tools::ToolData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    title: String,
    link: String,
    snippet: String,
}

impl Article {
    pub fn new(
        title: impl Into<String>,
        link: impl Into<String>,
        snippet: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            link: link.into(),
            snippet: snippet.into(),
        }
    }
}

impl Display for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]({})\n{}", self.title, self.link, self.snippet)
    }
}

impl From<Article> for ToolData {
    fn from(article: Article) -> Self {
        ToolData::Text(article.to_string())
    }
}
