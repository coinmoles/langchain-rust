use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{
    chain::{Chain, ChainError, ChainOutput, Ctor, InputCtor},
    output_parser::{parse_partial_json, OutputParseError},
    schemas::WithUsage,
};

pub struct PureOutput<O: DeserializeOwned + Send + Sync + 'static>(pub O);

impl<O: DeserializeOwned + Send + Sync + 'static> PureOutput<O> {
    pub fn into_inner(self) -> O {
        self.0
    }

    pub fn as_inner(&self) -> &O {
        &self.0
    }
}

impl<T, O: DeserializeOwned + Send + Sync> ChainOutput<T> for PureOutput<O> {
    fn construct_from_text(output: impl Into<String>) -> Result<Self, OutputParseError> {
        let original: String = output.into();
        let value = parse_partial_json(&original, false)?;
        let deserialized = serde_json::from_value::<O>(value)?;
        Ok(PureOutput(deserialized))
    }
}

pub struct PureOutputCtor<O: DeserializeOwned + Send + Sync>(std::marker::PhantomData<O>);

impl<O: DeserializeOwned + Send + Sync + 'static> Ctor for PureOutputCtor<O> {
    type Target<'a> = PureOutput<O>;
}

#[async_trait]
pub trait PureChain<I: InputCtor, O: DeserializeOwned + Send + Sync + 'static>:
    Chain<I, PureOutputCtor<O>>
{
    async fn call_pure(&self, input: I::Target<'_>) -> Result<WithUsage<O>, ChainError> {
        let WithUsage { content, usage } = self.call(input).await?;
        let content = content.into_inner();
        Ok(WithUsage { content, usage })
    }
}
