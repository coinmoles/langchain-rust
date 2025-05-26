#![allow(dead_code)]
// I have no idea how to remove dead codes here.

use std::{collections::HashSet, error::Error, pin::Pin};

use async_trait::async_trait;
use futures::Stream;

use crate::{
    chain::{load_stuff_qa, Chain, ChainError, LLMChain},
    language_models::llm::LLM,
    schemas::{GenerateResult, InputVariables, Prompt, StreamData},
};

use super::{
    StuffDocumentBuilder, COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME,
    COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY, STUFF_DOCUMENTS_DEFAULT_SEPARATOR,
};

pub struct StuffDocument {
    llm_chain: LLMChain,
    input_key: String,
    document_variable_name: String,
    separator: String,
}

impl StuffDocument {
    pub fn builder<'b>() -> StuffDocumentBuilder<'b> {
        StuffDocumentBuilder::new()
    }

    pub fn new(llm_chain: LLMChain) -> Self {
        Self {
            llm_chain,
            input_key: COMBINE_DOCUMENTS_DEFAULT_INPUT_KEY.into(),
            document_variable_name: COMBINE_DOCUMENTS_DEFAULT_DOCUMENT_VARIABLE_NAME.into(),
            separator: STUFF_DOCUMENTS_DEFAULT_SEPARATOR.into(),
        }
    }

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
        load_stuff_qa(llm)
    }
}

#[async_trait]
impl Chain for StuffDocument {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        self.llm_chain.call(input_variables).await
    }

    async fn stream(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.llm_chain.stream(input_variables).await
    }

    fn get_input_keys(&self) -> HashSet<String> {
        [self.input_key.clone()].into_iter().collect()
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>> {
        self.llm_chain.get_prompt(inputs)
    }
}
