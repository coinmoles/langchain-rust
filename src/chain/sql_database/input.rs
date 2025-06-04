use std::borrow::Cow;

use crate::schemas::{ChainInput, ChainInputCtor};

#[derive(Clone, Default, ChainInput, ChainInputCtor)]
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

#[derive(Clone, ChainInput, ChainInputCtor)]
pub struct SqlChainLLMChainInput<'a> {
    #[input(text)]
    pub input: Cow<'a, str>,
    #[input(text)]
    pub top_k: usize,
    #[input(text)]
    pub dialect: Cow<'a, str>,
    #[input(text)]
    pub tables_info: Cow<'a, str>,
}
