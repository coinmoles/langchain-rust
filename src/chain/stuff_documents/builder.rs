use crate::{
    chain::{ChainError, LLMChain},
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::MessageType,
    template::{MessageTemplate, PromptTemplate},
};

use super::{prompt::DEFAULT_STUFF_QA_TEMPLATE, StuffDocument};

pub struct StuffDocumentBuilder<'b> {
    llm: Option<Box<dyn LLM>>,
    output_key: Option<&'b str>,
    output_parser: Option<Box<dyn OutputParser>>,
    prompt: Option<PromptTemplate>,
}

impl<'b> StuffDocumentBuilder<'b> {
    pub(super) fn new() -> Self {
        Self {
            llm: None,
            output_key: None,
            output_parser: None,
            prompt: None,
        }
    }

    pub fn llm(mut self, llm: impl Into<Box<dyn LLM>>) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn output_key(mut self, output_key: &'b (impl AsRef<str> + ?Sized)) -> Self {
        self.output_key = Some(output_key.as_ref());
        self
    }

    ///If you want to add a custom prompt,keep in mind which variables are obligatory.
    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn build(self) -> Result<StuffDocument, ChainError> {
        let llm = self
            .llm
            .ok_or_else(|| ChainError::MissingObject("LLM must be set".into()))?;
        let prompt = match self.prompt {
            Some(prompt) => prompt,
            None => {
                MessageTemplate::from_fstring(MessageType::SystemMessage, DEFAULT_STUFF_QA_TEMPLATE)
                    .into()
            }
        };

        let llm_chain = {
            let mut builder = LLMChain::builder().prompt(prompt).llm(llm);
            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            builder.build()?
        };

        Ok(StuffDocument::new(llm_chain))
    }
}
