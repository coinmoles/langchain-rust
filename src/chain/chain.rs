use std::{borrow::Cow, pin::Pin};

use async_trait::async_trait;
use futures::Stream;

use crate::schemas::{
    ChainInputCtor, OutputTrace, ChainOutput, Prompt, StreamData, WithUsage,
};

use super::ChainError;

mod sealed {
    pub trait Sealed {}
}

#[async_trait]
pub trait ChainImpl: Send + Sync {
    type InputCtor: ChainInputCtor;
    type Output: ChainOutput;

    async fn call_impl<'i>(
        &self,
        input: Cow<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<WithUsage<Self::Output>, ChainError>;

    async fn call_with_trace_impl<'i>(
        &self,
        input: Cow<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<OutputTrace<Self::Output>, ChainError> {
        let output = self.call_impl(input).await?;

        Ok(OutputTrace::single(output))
    }

    async fn stream_impl<'i>(
        &self,
        _input: Cow<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        log::warn!("stream not implemented for this chain");
        unimplemented!()
    }

    fn get_prompt_impl<'i>(
        &self,
        input: Cow<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<Prompt, ChainError>;
}

#[async_trait]
pub trait Chain: ChainImpl + sealed::Sealed + Sync + Send {
    async fn call<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<WithUsage<Self::Output>, ChainError>;

    async fn call_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<WithUsage<Self::Output>, ChainError>;

    async fn call_with_trace<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<OutputTrace<Self::Output>, ChainError>;

    async fn call_with_trace_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<OutputTrace<Self::Output>, ChainError>;

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
    async fn stream<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>;

    async fn stream_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>;

    fn get_prompt<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Prompt, ChainError>;

    fn get_prompt_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Prompt, ChainError>;
}

impl<T> sealed::Sealed for T where T: ChainImpl {}

#[async_trait]
impl<T> Chain for T
where
    T: ChainImpl + sealed::Sealed,
{
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
    async fn call<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<WithUsage<Self::Output>, ChainError> {
        self.call_impl(Cow::Borrowed(input)).await
    }

    async fn call_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<WithUsage<Self::Output>, ChainError> {
        self.call_impl(Cow::Owned(input)).await
    }

    async fn call_with_trace<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<OutputTrace<Self::Output>, ChainError> {
        self.call_with_trace_impl(Cow::Borrowed(input)).await
    }

    async fn call_with_trace_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<OutputTrace<Self::Output>, ChainError> {
        self.call_with_trace_impl(Cow::Owned(input)).await
    }

    async fn stream<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.stream_impl(Cow::Borrowed(input)).await
    }

    async fn stream_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.stream_impl(Cow::Owned(input)).await
    }

    fn get_prompt<'i>(
        &self,
        input: &'i <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Prompt, ChainError> {
        self.get_prompt_impl(Cow::Borrowed(input))
    }

    fn get_prompt_owned<'i>(
        &self,
        input: <Self::InputCtor as ChainInputCtor>::Target<'i>,
    ) -> Result<Prompt, ChainError> {
        self.get_prompt_impl(Cow::Owned(input))
    }
}
