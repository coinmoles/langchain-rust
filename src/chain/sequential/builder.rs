use crate::{
    chain::Chain,
    schemas::{AsInput, ChainInputCtor},
};

use super::SequentialChain;

pub struct SequentialChainBuilder;

impl SequentialChainBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SequentialChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub trait AddChain<'a, I, Op>: 'a
where
    I: ChainInputCtor,
    Op: Chain + 'a,
{
    fn add_chain(self, chain: Op) -> impl Chain<InputCtor = I, Output = Op::Output> + 'a;
}

impl<'a, Op> AddChain<'a, Op::InputCtor, Op> for SequentialChainBuilder
where
    Op: Chain + 'a,
{
    fn add_chain(
        self,
        chain: Op,
    ) -> impl Chain<InputCtor = Op::InputCtor, Output = Op::Output> + 'a {
        chain
    }
}

impl<'a, Op1, Op2> AddChain<'a, Op1::InputCtor, Op2> for Op1
where
    Op1: Chain + 'a,
    Op2: Chain + 'a,
    Op1::Output: AsInput,
    for<'b> Op2::InputCtor: ChainInputCtor<Target<'b> = <Op1::Output as AsInput>::AsInput<'b>>,
{
    fn add_chain(
        self,
        chain: Op2,
    ) -> impl Chain<InputCtor = Op1::InputCtor, Output = Op2::Output> + 'a {
        SequentialChain {
            first: Box::new(self),
            second: Box::new(chain),
        }
    }
}
