use crate::{
    chain::LLMChain,
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::BuilderError,
    schemas::{ChainOutput, InputCtor, MessageType, OutputCtor},
    template::{MessageTemplate, PromptTemplate},
};

use super::{prompt::DEFAULT_STUFF_QA_TEMPLATE, StuffDocument};

pub struct StuffDocumentBuilder<'a, I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    llm: Option<Box<dyn LLM>>,
    output_key: Option<&'a str>,
    output_parser: Option<Box<dyn OutputParser>>,
    prompt: Option<PromptTemplate>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'a, I, O> StuffDocumentBuilder<'a, I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) fn new() -> Self {
        Self {
            llm: None,
            output_key: None,
            output_parser: None,
            prompt: None,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn llm(mut self, llm: impl Into<Box<dyn LLM>>) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn output_key(mut self, output_key: &'a (impl AsRef<str> + ?Sized)) -> Self {
        self.output_key = Some(output_key.as_ref());
        self
    }

    ///If you want to add a custom prompt,keep in mind which variables are obligatory.
    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn build(self) -> Result<StuffDocument<I, O>, BuilderError> {
        let llm = self.llm.ok_or(BuilderError::MissingField("llm"))?;
        let prompt = match self.prompt {
            Some(prompt) => prompt,
            None => {
                MessageTemplate::from_fstring(MessageType::System, DEFAULT_STUFF_QA_TEMPLATE).into()
            }
        };

        let llm_chain = {
            let mut builder = LLMChain::builder().prompt(prompt).llm(llm);
            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            builder
                .build()
                .map_err(|e| BuilderError::Inner("llm_chain", Box::new(e)))?
        };

        Ok(StuffDocument::new(llm_chain))
    }
}
