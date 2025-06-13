use std::{borrow::Cow, marker::PhantomData};

use crate::chain::{ChainInput, Ctor, DefaultChainInput, DefaultChainInputCtor, InputCtor};

pub struct ConversationalChainInputCtor<I: InputCtor = DefaultChainInputCtor>(PhantomData<I>);
impl<I: InputCtor> Ctor for ConversationalChainInputCtor<I> {
    type Target<'a> = ConversationalChainInput<'a, I::Target<'a>>;
}

#[derive(Clone, ChainInput)]
pub struct ConversationalChainInput<'a, I: ChainInput = DefaultChainInput<'a>> {
    #[langchain(into = "inner")]
    pub inner: I,
    #[langchain(into = "text")]
    pub chat_history: Option<Cow<'a, str>>,
}

impl<'a, I> ConversationalChainInput<'a, I>
where
    I: ChainInput,
{
    pub fn new(input: I) -> Self {
        Self {
            inner: input,
            chat_history: None,
        }
    }

    pub fn with_history(mut self, chat_history: impl Into<Cow<'a, str>>) -> Self {
        self.chat_history = Some(chat_history.into());
        self
    }
}
