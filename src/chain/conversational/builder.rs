use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{
    chain::{ChainError, LLMChain, DEFAULT_OUTPUT_KEY},
    language_models::llm::LLM,
    memory::{Memory, SimpleMemory},
    output_parsers::OutputParser,
    schemas::MessageType,
    template::{MessageTemplate, PromptTemplate},
};

use super::{prompt::DEFAULT_TEMPLATE, ConversationalChain, DEFAULT_INPUT_VARIABLE};

pub struct ConversationalChainBuilder<'a, 'b> {
    llm: Option<Box<dyn LLM>>,
    memory: Option<Arc<RwLock<dyn Memory>>>,
    input_key: Option<&'a str>,
    output_key: Option<&'b str>,
    output_parser: Option<Box<dyn OutputParser>>,
    prompt: Option<PromptTemplate>,
}

impl<'a, 'b> ConversationalChainBuilder<'a, 'b> {
    pub(super) fn new() -> Self {
        Self {
            llm: None,
            memory: None,
            input_key: None,
            output_key: None,
            output_parser: None,
            prompt: None,
        }
    }

    pub fn llm(mut self, llm: impl Into<Box<dyn LLM>>) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn memory(mut self, memory: Arc<RwLock<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn input_key(mut self, input_key: &'a (impl AsRef<str> + ?Sized)) -> Self {
        self.input_key = Some(input_key.as_ref());
        self
    }

    pub fn output_key(mut self, output_key: &'b (impl AsRef<str> + ?Sized)) -> Self {
        self.output_key = Some(output_key.as_ref());
        self
    }

    pub fn output_parser(mut self, output_parser: impl Into<Box<dyn OutputParser>>) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    ///If you want to add a custom prompt,keep in mind which variables are obligatory.
    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn build(self) -> Result<ConversationalChain, ChainError> {
        let llm = self
            .llm
            .ok_or_else(|| ChainError::MissingObject("LLM must be set".into()))?;
        let prompt = match self.prompt {
            Some(prompt) => prompt,
            None => {
                MessageTemplate::from_fstring(MessageType::HumanMessage, DEFAULT_TEMPLATE).into()
            }
        };
        let llm_chain = {
            let b = self.output_key.unwrap_or(DEFAULT_OUTPUT_KEY);

            let mut builder = LLMChain::builder().prompt(prompt).llm(llm).output_key(b);

            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            builder.build()?
        };

        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(RwLock::new(SimpleMemory::new())));

        Ok(ConversationalChain {
            llm_chain,
            memory,
            input_key: self.input_key.unwrap_or(DEFAULT_INPUT_VARIABLE).into(),
        })
    }
}
