use std::{borrow::Cow, marker::PhantomData};

use crate::schemas::{DefaultChainInput, ChainInput, ChainInputCtor, TextReplacements};

pub struct ConversationalChainInputCtor<I>(PhantomData<I>)
where
    I: ChainInputCtor;

impl<I> ChainInputCtor for ConversationalChainInputCtor<I>
where
    I: ChainInputCtor,
{
    type Target<'a> = ConversationalChainInput<'a, I::Target<'a>>;
}

#[derive(Clone)]
pub struct ConversationalChainInput<'a, I = DefaultChainInput<'a>>
where
    I: ChainInput,
{
    pub inner: Cow<'a, I>,
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

impl<'a, I> ChainInput for ConversationalChainInput<'a, I>
where
    I: ChainInput,
{
    fn text_replacements(&self) -> TextReplacements {
        let mut replacements = self.inner.text_replacements();
        replacements.insert("history", self.chat_history.as_deref().unwrap_or("").into());
        replacements
    }
}
