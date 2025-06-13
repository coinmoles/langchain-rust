use async_trait::async_trait;

use crate::{
    chain::{Chain, ChainError},
    schemas::{IntoWithUsage, WithUsage},
};

pub struct EmptyChain;

#[async_trait]
impl Chain<(), ()> for EmptyChain {
    async fn call<'a>(&self, _input: ()) -> Result<WithUsage<()>, ChainError> {
        Ok(().with_usage(None))
    }
}
