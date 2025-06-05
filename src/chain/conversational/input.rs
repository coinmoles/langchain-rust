use std::{borrow::Cow, marker::PhantomData};

use crate::schemas::{ChainInput, ChainInputCtor, DefaultChainInput, DefaultChainInputCtor};

pub struct ConversationalChainInputCtor<I: ChainInputCtor = DefaultChainInputCtor>(PhantomData<I>);
impl<I: ChainInputCtor> ChainInputCtor for ConversationalChainInputCtor<I> {
    type Target<'a> = ConversationalChainInput<'a, I::Target<'a>>;
}

#[derive(Clone, ChainInput)]
pub struct ConversationalChainInput<'a, I: ChainInput = DefaultChainInput<'a>> {
    #[chain_input(inner)]
    pub inner: Cow<'a, I>,
    #[chain_input(text)]
    pub chat_history: Option<Cow<'a, str>>,
}

impl<'a, I> ConversationalChainInput<'a, I>
where
    I: ChainInput,
{
    pub fn new(input: impl Into<Cow<'a, I>>) -> Self {
        Self {
            inner: input.into(),
            chat_history: None,
        }
    }

    pub fn with_history(mut self, chat_history: impl Into<Cow<'a, str>>) -> Self {
        self.chat_history = Some(chat_history.into());
        self
    }
}
