use serde::Serialize;

use crate::{
    chain::Chain,
    schemas::{Ctor, InputCtor},
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
    I: InputCtor,
    Op: Chain + 'a,
{
    fn add_chain(self, chain: Op) -> impl Chain<InputCtor = I, OutputCtor = Op::OutputCtor> + 'a;
}

impl<'a, Op> AddChain<'a, Op::InputCtor, Op> for SequentialChainBuilder
where
    Op: Chain + 'a,
{
    fn add_chain(
        self,
        chain: Op,
    ) -> impl Chain<InputCtor = Op::InputCtor, OutputCtor = Op::OutputCtor> + 'a {
        chain
    }
}

impl<'a, Op1, Op2> AddChain<'a, Op1::InputCtor, Op2> for Op1
where
    Op1: Chain + 'a,
    Op2: Chain + 'a,
    for<'b> <Op1::OutputCtor as Ctor>::Target<'b>:
        Serialize + Clone + Into<<Op2::InputCtor as InputCtor>::Target<'b>>,
    for<'b> <Op2::OutputCtor as Ctor>::Target<'b>: Serialize,
{
    fn add_chain(
        self,
        chain: Op2,
    ) -> impl Chain<InputCtor = Op1::InputCtor, OutputCtor = Op2::OutputCtor> + 'a {
        SequentialChain {
            first: Box::new(self),
            second: Box::new(chain),
        }
    }
}
