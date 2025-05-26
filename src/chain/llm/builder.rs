use crate::{
    chain::ChainError,
    language_models::llm::LLM,
    output_parsers::{OutputParser, SimpleParser},
    template::PromptTemplate,
};

use super::LLMChain;

pub struct LLMChainBuilder<'b> {
    prompt: Option<PromptTemplate>,
    llm: Option<Box<dyn LLM>>,
    output_key: Option<&'b str>,
    output_parser: Option<Box<dyn OutputParser>>,
}

impl<'b> LLMChainBuilder<'b> {
    pub(super) fn new() -> Self {
        Self {
            prompt: None,
            llm: None,
            output_key: None,
            output_parser: None,
        }
    }

    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn llm(mut self, llm: impl Into<Box<dyn LLM>>) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn output_key(mut self, output_key: &'b (impl AsRef<str> + ?Sized)) -> Self {
        self.output_key = Some(output_key.as_ref());
        self
    }

    pub fn output_parser<P: Into<Box<dyn OutputParser>>>(mut self, output_parser: P) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    pub fn build(self) -> Result<LLMChain, ChainError> {
        let prompt = self
            .prompt
            .ok_or_else(|| ChainError::MissingObject("Prompt must be set".into()))?;

        let llm = self
            .llm
            .ok_or_else(|| ChainError::MissingObject("LLM must be set".into()))?;

        let chain = LLMChain {
            prompt,
            llm,
            output_parser: self
                .output_parser
                .unwrap_or_else(|| Box::new(SimpleParser::default())),
        };

        Ok(chain)
    }
}
