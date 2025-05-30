use std::{error::Error, pin::Pin};

use async_trait::async_trait;
use futures::Stream;

use crate::schemas::{
    InputVariableCtor, OutputTrace, OutputVariable, Prompt, StreamData, WithUsage,
};

use super::ChainError;

pub(crate) const DEFAULT_OUTPUT_KEY: &str = "output";

#[async_trait]
pub trait Chain: Sync + Send {
    type InputCtor: InputVariableCtor;
    type Output: OutputVariable;

    /// Call the `Chain` and receive as output the result of the generation process along with
    /// additional information like token consumption.
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
    async fn call<'i, 'i2>(
        &self,
        input: &'i <Self::InputCtor as InputVariableCtor>::InputVariable<'i2>,
    ) -> Result<WithUsage<Self::Output>, ChainError>
    where
        'i: 'i2;

    async fn call_with_trace<'i, 'i2>(
        &self,
        input: &'i <Self::InputCtor as InputVariableCtor>::InputVariable<'i2>,
    ) -> Result<OutputTrace<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
        let output = self.call(&input).await?;

        Ok(OutputTrace::single(output))
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
    async fn stream<'i, 'i2>(
        &self,
        _input: &'i <Self::InputCtor as InputVariableCtor>::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        log::warn!("stream not implemented for this chain");
        unimplemented!()
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i <Self::InputCtor as InputVariableCtor>::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2;
}
