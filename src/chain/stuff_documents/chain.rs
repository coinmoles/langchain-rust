#![allow(dead_code)]
// I have no idea how to remove dead codes here.

use std::{borrow::Cow, pin::Pin};

use async_trait::async_trait;
use futures::Stream;
use indoc::indoc;

use crate::{
    chain::{ChainError, ChainImpl, LLMChain, StuffQACtor},
    language_models::llm::LLM,
    schemas::{ChainInputCtor, MessageType, ChainOutput, Prompt, StreamData, WithUsage},
    template::MessageTemplate,
};

use super::{
    StuffDocumentBuilder, COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME,
    COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY, STUFF_DOCUMENTS_DEFAULT_SEPARATOR,
};

pub struct StuffDocument<I, O = String>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    llm_chain: LLMChain<I, O>,
    input_key: String,
    document_variable_name: String,
    separator: String,
}

impl<I, O> StuffDocument<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
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
impl<I, O> ChainImpl for StuffDocument<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    type InputCtor = I;
    type Output = O;

    async fn call_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<WithUsage<O>, ChainError> {
        self.llm_chain.call_impl(input).await
    }

    async fn stream_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.llm_chain.stream_impl(input).await
    }

    fn get_prompt_impl<'i>(&self, input: Cow<'i, I::Target<'i>>) -> Result<Prompt, ChainError> {
        self.llm_chain.get_prompt_impl(input)
    }
}
