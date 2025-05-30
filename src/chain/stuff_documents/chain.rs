#![allow(dead_code)]
// I have no idea how to remove dead codes here.

use std::{error::Error, pin::Pin};

use async_trait::async_trait;
use futures::Stream;
use indoc::indoc;

use crate::{
    chain::{Chain, ChainError, LLMChain, StuffQACtor},
    language_models::llm::LLM,
    schemas::{InputVariableCtor, MessageType, OutputVariable, Prompt, StreamData, WithUsage},
    template::MessageTemplate,
};

use super::{
    StuffDocumentBuilder, COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME,
    COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY, STUFF_DOCUMENTS_DEFAULT_SEPARATOR,
};

pub struct StuffDocument<I, O = String>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    llm_chain: LLMChain<I, O>,
    input_key: String,
    document_variable_name: String,
    separator: String,
}

impl<I, O> StuffDocument<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    pub fn builder<'b>() -> StuffDocumentBuilder<'b, I, O> {
        StuffDocumentBuilder::new()
    }

    pub fn new(llm_chain: LLMChain<I, O>) -> Self {
        Self {
            llm_chain,
            input_key: COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY.into(),
            document_variable_name: COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME.into(),
            separator: STUFF_DOCUMENTS_DEFAULT_SEPARATOR.into(),
        }
    }
}

impl StuffDocument<StuffQACtor, String> {
    /// load_stuff_qa return an instance of StuffDocument
    /// with a prompt desiged for question ansering
    ///
    /// # Example
    /// ```rust,ignore
    ///
    /// let llm = OpenAI::default();
    /// let chain = StuffDocument::load_stuff_qa(llm);
    ///
    /// let input = chain
    /// .qa_prompt_builder()
    /// .documents(&[
    /// Document::new(indoc! {"
    ///     Question: Which is the favorite text editor of luis
    ///     Answer: Nvim"
    /// }),
    /// Document::new(indoc! {"
    ///    Question: How old is luis
    ///    Answer: 24"
    /// }),
    /// ])
    /// .question("How old is luis and whats his favorite text editor")
    /// .build();
    ///
    /// let ouput = chain.invoke(input).await.unwrap();
    ///
    /// println!("{}", ouput);
    /// ```
    ///
    pub fn load_stuff_qa<L: Into<Box<dyn LLM>>>(llm: L) -> Self {
        let default_qa_prompt_template = MessageTemplate::from_jinja2(
            MessageType::SystemMessage,
            indoc! {"
            Use the following pieces of context to answer the question at the end. If you don't know the answer, just say that you don't know, don't try to make up an answer.
            
            {{context}}
            
            Question:{{question}}
            Helpful Answer:"},
        );

        let llm_chain_builder = LLMChain::builder()
            .prompt(default_qa_prompt_template)
            .llm(llm)
            .build()
            .unwrap();

        let llm_chain = llm_chain_builder;

        StuffDocument::new(llm_chain)
    }
}

#[async_trait]
impl<I, O> Chain for StuffDocument<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    type InputCtor = I;
    type Output = O;

    async fn call<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<WithUsage<O>, ChainError>
    where
        'i: 'i2,
    {
        self.llm_chain.call(input).await
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        self.llm_chain.stream(input).await
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2,
    {
        self.llm_chain.get_prompt(input)
    }
}
