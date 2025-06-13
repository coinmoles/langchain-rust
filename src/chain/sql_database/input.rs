use std::borrow::Cow;

use crate::chain::{ChainInput, Ctor};

#[derive(Clone, Default, ChainInput, Ctor)]
pub struct SqlChainInput<'a> {
    pub query: &'a str,
    pub tables: &'a [String],
}

impl<'a> SqlChainInput<'a> {
    pub fn new(query: &'a str, tables: &'a [String]) -> Self {
        Self { query, tables }
    }

    pub fn query(mut self, query: &'a str) -> Self {
        self.query = query;
        self
    }

    pub fn tables(mut self, tables: &'a [String]) -> Self {
        self.tables = tables;
        self
    }
}

#[derive(Clone, ChainInput, Ctor)]
pub struct SqlChainLLMChainInput<'a> {
    #[langchain(into = "text")]
    pub input: Cow<'a, str>,
    #[langchain(into = "text")]
    pub top_k: usize,
    #[langchain(into = "text")]
    pub dialect: Cow<'a, str>,
    #[langchain(into = "text")]
    pub tables_info: Cow<'a, str>,
}
