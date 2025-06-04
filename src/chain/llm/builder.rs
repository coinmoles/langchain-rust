use crate::{
    chain::ChainError,
    language_models::llm::LLM,
    output_parsers::{OutputParser, SimpleParser},
    schemas::{ChainInputCtor, ChainOutput},
    template::PromptTemplate,
};

use super::LLMChain;

pub struct LLMChainBuilder<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    prompt: Option<PromptTemplate>,
    llm: Option<Box<dyn LLM>>,
    output_parser: Option<Box<dyn OutputParser>>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> LLMChainBuilder<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    pub(super) fn new() -> Self {
        Self {
            prompt: None,
            llm: None,
            output_parser: None,
            _phantom: std::marker::PhantomData,
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

    pub fn output_parser<P: Into<Box<dyn OutputParser>>>(mut self, output_parser: P) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    pub fn build(self) -> Result<LLMChain<I, O>, ChainError> {
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
            _phantom: std::marker::PhantomData,
        };

        Ok(chain)
    }
}
