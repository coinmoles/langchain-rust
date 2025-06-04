use std::{borrow::Cow, collections::HashMap};

use crate::schemas::{ChainInput, ChainInputCtor, TextReplacements};

pub struct SqlChainInputCtor;
impl ChainInputCtor for SqlChainInputCtor {
    type Target<'a> = SqlChainInput<'a>;
}

#[derive(Clone, Default)]
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

impl ChainInput for SqlChainInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::new()
    }
}

pub struct SqlChainLLMChainInputCtor;
impl ChainInputCtor for SqlChainLLMChainInputCtor {
    type Target<'a> = SqlChainLLMChainInput<'a>;
}

#[derive(Clone)]
pub struct SqlChainLLMChainInput<'a> {
    pub input: Cow<'a, str>,
    pub top_k: usize,
    pub dialect: Cow<'a, str>,
    pub tables_info: Cow<'a, str>,
}

impl ChainInput for SqlChainLLMChainInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("input", self.input.as_ref().into()),
            ("top_k", self.top_k.to_string().into()),
            ("dialect", self.dialect.as_ref().into()),
            ("table_info", self.tables_info.as_ref().into()),
        ])
    }
}
