use crate::{
    chain::{ChainOutput, InputCtor, OutputCtor},
    llm::LLM,
    output_parser::{OutputParser, SimpleParser},
    schemas::BuilderError,
    template::PromptTemplate,
};

use super::LLMChain;

pub struct LLMChainBuilder<I: InputCtor, O: OutputCtor>
where
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    prompt: Option<PromptTemplate>,
    llm: Option<Box<dyn LLM>>,
    output_parser: Option<Box<dyn OutputParser<I, O>>>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: InputCtor, O: OutputCtor> LLMChainBuilder<I, O>
where
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
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

    pub fn output_parser<P: Into<Box<dyn OutputParser<I, O>>>>(mut self, output_parser: P) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    pub fn build(self) -> Result<LLMChain<I, O>, BuilderError> {
        let prompt = self.prompt.ok_or(BuilderError::MissingField("prompt"))?;
        let llm = self.llm.ok_or(BuilderError::MissingField("llm"))?;
        let output_parser = self
            .output_parser
            .unwrap_or_else(|| Box::new(SimpleParser::default()));

        let chain = LLMChain {
            prompt,
            llm,
            output_parser,
            _phantom: std::marker::PhantomData,
        };

        Ok(chain)
    }
}
