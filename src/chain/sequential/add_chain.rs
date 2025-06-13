use serde::Serialize;

use crate::{
    chain::Chain,
    schemas::{InputCtor, OutputCtor},
};

use super::SequentialChain;

pub trait AddChain<'a, I, M1, M2, O>: 'a
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
{
    fn add_chain(self, chain: impl Chain<M2, O> + 'a) -> impl Chain<I, O> + 'a;
}

impl<'a, I, M1, M2, O, Op> AddChain<'a, I, M1, M2, O> for Op
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    Op: Chain<I, M1> + 'a,
    for<'any> M1::Target<'any>: Serialize + Clone + Into<M2::Target<'any>>,
    for<'any> O::Target<'any>: Serialize,
{
    fn add_chain(self, chain: impl Chain<M2, O> + 'a) -> impl Chain<I, O> + 'a {
        SequentialChain {
            first: Box::new(self),
            second: Box::new(chain),
        }
    }
}
