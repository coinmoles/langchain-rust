use std::{fmt::Display, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    chain::{ConversationalChainInputCtor, LLMChain},
    language_models::llm::LLM,
    memory::{Memory, SimpleMemory},
    output_parser::OutputParser,
    schemas::{BuilderError, ChainOutput, InputCtor, LLMOutputCtor, MessageType, OutputCtor},
    template::{MessageTemplate, PromptTemplate},
};

use super::{prompt::DEFAULT_TEMPLATE, ConversationalChain};

pub struct ConversationalChainBuilder<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    llm: Option<Box<dyn LLM>>,
    memory: Option<Arc<RwLock<dyn Memory>>>,
    output_parser: Option<Box<dyn OutputParser<ConversationalChainInputCtor<I>, LLMOutputCtor>>>,
    prompt: Option<PromptTemplate>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> ConversationalChainBuilder<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
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

    pub fn output_parser(
        mut self,
        output_parser: impl Into<Box<dyn OutputParser<ConversationalChainInputCtor<I>, LLMOutputCtor>>>,
    ) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    ///If you want to add a custom prompt,keep in mind which variables are obligatory.
    pub fn prompt(mut self, prompt: impl Into<PromptTemplate>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn build(self) -> Result<ConversationalChain<I, O>, BuilderError> {
        let llm = self.llm.ok_or(BuilderError::MissingField("llm"))?;
        let prompt = match self.prompt {
            Some(prompt) => prompt,
            None => MessageTemplate::from_fstring(MessageType::Human, DEFAULT_TEMPLATE).into(),
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

        let memory = self
            .memory
            .unwrap_or_else(|| Arc::new(RwLock::new(SimpleMemory::new())));

        Ok(ConversationalChain {
            llm_chain,
            memory,
            _phantom: std::marker::PhantomData,
        })
    }
}
