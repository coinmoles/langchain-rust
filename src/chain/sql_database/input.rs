use std::{borrow::Cow, collections::HashMap};

use crate::schemas::{InputVariable, InputVariableCtor, TextReplacements};

pub struct SqlChainDefaultInputCtor;
impl InputVariableCtor for SqlChainDefaultInputCtor {
    type InputVariable<'a> = SqlChainDefaultInput<'a>;
}

pub struct SqlChainDefaultInput<'a> {
    pub query: &'a str,
    pub tables: &'a [String],
}

impl InputVariable for SqlChainDefaultInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::new()
    }
}

pub struct SqlChainLLMChainInputCtor;
impl InputVariableCtor for SqlChainLLMChainInputCtor {
    type InputVariable<'a> = SqlChainLLMChainInput<'a>;
}

pub struct SqlChainLLMChainInput<'a> {
    pub input: Cow<'a, str>,
    pub top_k: usize,
    pub dialect: Cow<'a, str>,
    pub tables_info: Cow<'a, str>,
}

impl InputVariable for SqlChainLLMChainInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("input", self.input.as_ref().into()),
            ("top_k", self.top_k.to_string().into()),
            ("dialect", self.dialect.as_ref().into()),
            ("table_info", self.tables_info.as_ref().into()),
        ])
    }
}
