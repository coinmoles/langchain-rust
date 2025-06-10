#![allow(dead_code)]
// I have no idea how to remove dead codes here.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use indoc::indoc;

use crate::{
    chain::{Chain, ChainError, LLMChain, StuffQACtor},
    language_models::llm::LLM,
    schemas::{
        ChainOutput, GetPrompt, InputCtor, MessageType, OutputCtor, Prompt, StreamData, StringCtor,
        WithUsage,
    },
    template::{MessageTemplate, TemplateError},
};

use super::{
    StuffDocumentBuilder, COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME,
    COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY, STUFF_DOCUMENTS_DEFAULT_SEPARATOR,
};

pub struct StuffDocument<I = StuffQACtor, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    llm_chain: LLMChain<I, O>,
    input_key: String,
    document_variable_name: String,
    separator: String,
}

impl<I, O> StuffDocument<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn builder<'any>() -> StuffDocumentBuilder<'any, I, O> {
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

impl StuffDocument<StuffQACtor, StringCtor> {
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
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        self.llm_chain.call(input).await
    }

    async fn stream(
        &self,
        input: I::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.llm_chain.stream(input).await
    }
}

impl<I, O> GetPrompt<I::Target<'_>> for StuffDocument<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: &I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }
}
