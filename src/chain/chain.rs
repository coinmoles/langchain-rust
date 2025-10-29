use async_trait::async_trait;

use super::ChainError;
use crate::chain::{InputCtor, OutputCtor};
use crate::schemas::{OutputTrace, WithUsage};

#[async_trait]
pub trait Chain<I: InputCtor, O: OutputCtor>: Sync + Send {
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError>;

    async fn call_with_trace<'a>(
        &self,
        input: I::Target<'a>,
    ) -> Result<OutputTrace<O::Target<'a>>, ChainError> {
        let output = self.call(input).await?;

        Ok(OutputTrace::single(output))
    }
}
