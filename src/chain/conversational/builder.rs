use std::{fmt::Display, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    chain::{ChainError, ConversationalChainInput, LLMChain},
    language_models::llm::LLM,
    memory::{Memory, SimpleMemory},
    output_parsers::OutputParser,
    schemas::{ChainOutput, Ctor, InputCtor, MessageType},
    template::{MessageTemplate, PromptTemplate},
};

use super::{prompt::DEFAULT_TEMPLATE, ConversationalChain};

pub struct ConversationalChainBuilder<I, O>
where
    I: InputCtor,
    O: Ctor,
    for<'b> I::Target<'b>: Display,
    for<'b> O::Target<'b>: ChainOutput<ConversationalChainInput<'b, I::Target<'b>>>,
{
    llm: Option<Box<dyn LLM>>,
    memory: Option<Arc<RwLock<dyn Memory>>>,
    output_parser: Option<Box<dyn OutputParser>>,
    prompt: Option<PromptTemplate>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> ConversationalChainBuilder<I, O>
where
    I: InputCtor,
    O: Ctor,
    for<'b> I::Target<'b>: Display,
    for<'b> O::Target<'b>: ChainOutput<ConversationalChainInput<'b, I::Target<'b>>>,
{
    pub(super) fn new() -> Self {
        Self {
            llm: None,
            memory: None,
            output_parser: None,
            prompt: None,
            _phantom: std::marker::PhantomData,
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

    pub fn output_parser(mut self, output_parser: impl Into<Box<dyn OutputParser>>) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    ///If you want to add a custom prompt,keep in mind which variables are obligatory.
    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn build(self) -> Result<ConversationalChain<I, O>, ChainError> {
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
            let mut builder = LLMChain::builder().prompt(prompt).llm(llm);

            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            builder.build()?
        };

        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(RwLock::new(SimpleMemory::new())));

        Ok(ConversationalChain { llm_chain, memory })
    }
}
