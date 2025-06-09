use std::borrow::Cow;

use crate::schemas::{ChainInput, Ctor};

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
    #[chain_input(text)]
    pub input: Cow<'a, str>,
    #[chain_input(text)]
    pub top_k: usize,
    #[chain_input(text)]
    pub dialect: Cow<'a, str>,
    #[chain_input(text)]
    pub tables_info: Cow<'a, str>,
}
