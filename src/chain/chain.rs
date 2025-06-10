use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::schemas::{InputCtor, OutputCtor, OutputTrace, Prompt, StreamData, WithUsage};

use super::ChainError;

#[async_trait]
pub trait Chain: Sync + Send {
    type InputCtor: InputCtor;
    type OutputCtor: OutputCtor;

    async fn call<'a>(
        &self,
        input: <Self::InputCtor as InputCtor>::Target<'a>,
    ) -> Result<WithUsage<<Self::OutputCtor as OutputCtor>::Target<'a>>, ChainError>;

    async fn call_with_trace<'a>(
        &self,
        input: <Self::InputCtor as InputCtor>::Target<'a>,
    ) -> Result<OutputTrace<<Self::OutputCtor as OutputCtor>::Target<'a>>, ChainError> {
        let output = self.call(input).await?;

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
    async fn stream(
        &self,
        _input: <Self::InputCtor as InputCtor>::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        unimplemented!("Streaming is not implemented for this chain")
    }

    fn get_prompt(
        &self,
        input: <Self::InputCtor as InputCtor>::Target<'_>,
    ) -> Result<Prompt, ChainError>;
}
