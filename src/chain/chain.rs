use std::{
    collections::{HashMap, HashSet},
    error::Error,
    pin::Pin,
};

use async_trait::async_trait;
use futures::Stream;
use serde_json::Value;

use crate::schemas::{GenerateResult, InputVariables, Prompt, StreamData};

use super::ChainError;

pub(crate) const DEFAULT_OUTPUT_KEY: &str = "output";
pub(crate) const DEFAULT_RESULT_KEY: &str = "generate_result";

#[async_trait]
pub trait Chain: Sync + Send {
    /// Call the `Chain` and receive as output the result of the generation process along with
    /// additional information like token consumption. The input is a set of variables passed
    /// as a `PromptArgs` hashmap.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use crate::my_crate::{Chain, ConversationalChainBuilder, OpenAI, OpenAIModel, SimpleMemory, PromptArgs, prompt_args};
    /// # async {
    /// let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).builder();
    /// let memory = SimpleMemory::new();
    ///
    /// let chain = ConversationalChainBuilder::new()
    ///     .llm(llm)
    ///     .memory(memory.into())
    ///     .build().expect("Error building ConversationalChain");
    ///
    /// let input_variables = prompt_args! {
    ///     "input" => "Im from Peru",
    /// };
    ///
    /// match chain.call(input_variables).await {
    ///     Ok(result) => {
    ///         println!("Result: {:?}", result);
    ///     },
    ///     Err(e) => panic!("Error calling Chain: {:?}", e),
    /// };
    /// # };
    /// ```
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError>;

    /// Invoke the `Chain` and receive just the generation result as a String.
    /// The input is a set of variables passed as a `PromptArgs` hashmap.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use crate::my_crate::{Chain, ConversationalChainBuilder, OpenAI, OpenAIModel, SimpleMemory, PromptArgs, prompt_args};
    /// # async {
    /// let llm = OpenAI::default().with_model(OpenAIModel::Gpt35);
    /// let memory = SimpleMemory::new();
    ///
    /// let chain = ConversationalChainBuilder::new()
    ///     .llm(llm)
    ///     .memory(memory.into())
    ///     .build().expect("Error building ConversationalChain");
    ///
    /// let input_variables = prompt_args! {
    ///     "input" => "Im from Peru",
    /// };
    ///
    /// match chain.invoke(input_variables).await {
    ///     Ok(result) => {
    ///         println!("Result: {:?}", result);
    ///     },
    ///     Err(e) => panic!("Error invoking Chain: {:?}", e),
    /// };
    /// # };
    /// ```
    async fn invoke(&self, input_variables: &mut InputVariables) -> Result<String, ChainError> {
        self.call(input_variables)
            .await
            .map(|result| result.content.text().into())
    }

    /// Execute the `Chain` and return the result of the generation process
    /// along with additional information like token consumption formatted as a `HashMap`.
    /// The input is a set of variables passed as a `PromptArgs` hashmap.
    /// The key for the generated output is specified by the `get_output_keys`
    /// method (default key is `output`).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use crate::my_crate::{Chain, ConversationalChainBuilder, OpenAI, OpenAIModel, SimpleMemory, PromptArgs, prompt_args};
    /// # async {
    /// let llm = OpenAI::default().with_model(OpenAIModel::Gpt35);
    /// let memory = SimpleMemory::new();
    ///
    /// let chain = ConversationalChainBuilder::new()
    ///     .llm(llm)
    ///     .memory(memory.into())
    ///     .output_key("name")
    ///     .build().expect("Error building ConversationalChain");
    ///
    /// let input_variables = prompt_args! {
    ///     "input" => "Im from Peru",
    /// };
    ///
    /// match chain.execute(input_variables).await {
    ///     Ok(result) => {
    ///         println!("Result: {:?}", result);
    ///     },
    ///     Err(e) => panic!("Error executing Chain: {:?}", e),
    /// };
    /// # };
    /// ```
    /// BROKEN!!
    async fn execute(
        &self,
        _input_variables: &mut InputVariables,
    ) -> Result<HashMap<String, Value>, ChainError> {
        // let result = self.call(input_variables).await?;
        // let mut output = HashMap::new();
        // let output_key = self
        //     .get_output_keys()
        //     .first()
        //     .unwrap_or(&DEFAULT_OUTPUT_KEY.to_string())
        //     .clone();
        // output.insert(output_key, result.content);
        // output.insert(DEFAULT_RESULT_KEY.to_string(), json!(result));
        Ok(HashMap::new())
    }

    /// Stream the `Chain` and get an asynchronous stream of chain generations.
    /// The input is a set of variables passed as a `PromptArgs` hashmap.
    /// If the chain have memroy, the tream method will not be able to automaticaly
    /// set the memroy, bocause it will not know if the how to extract the output message
    /// out of the stram
    /// # Example
    ///
    /// ```rust,ignore
    /// # use futures::StreamExt;
    /// # use crate::my_crate::{Chain, LLMChainBuilder, OpenAI, fmt_message, fmt_template,
    /// #                      HumanMessagePromptTemplate, prompt_args, Message, template_fstring};
    /// # async {
    /// let open_ai = OpenAI::default();
    ///
    ///let prompt = message_formatter![
    ///fmt_message!(Message::new_system_message(
    ///"You are world class technical documentation writer."
    ///)),
    ///fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
    ///      "{input}", "input"
    ///)))
    ///];
    ///
    /// let chain = LLMChainBuilder::new()
    ///     .prompt(prompt)
    ///     .llm(open_ai.clone())
    ///     .build()
    ///     .unwrap();
    ///
    /// let mut stream = chain.stream(
    /// prompt_args! {
    /// "input" => "Who is the writer of 20,000 Leagues Under the Sea?"
    /// }).await.unwrap();
    ///
    /// while let Some(result) = stream.next().await {
    ///     match result {
    ///         Ok(value) => {
    ///                 println!("Content: {}", value.content);
    ///         },
    ///         Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    ///     }
    /// };
    /// # };
    /// ```
    ///
    async fn stream(
        &self,
        _input_variables: &mut InputVariables,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        log::warn!("stream not implemented for this chain");
        unimplemented!()
    }

    // Get the input keys of the prompt
    fn get_input_keys(&self) -> HashSet<String> {
        HashSet::new()
    }

    fn get_output_keys(&self) -> Vec<String> {
        [
            String::from(DEFAULT_OUTPUT_KEY),
            String::from(DEFAULT_RESULT_KEY),
        ]
        .into_iter()
        .collect()
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>>;
}
