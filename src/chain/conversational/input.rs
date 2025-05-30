use std::{borrow::Cow, marker::PhantomData};

use crate::schemas::{DefaultChainInput, InputVariable, InputVariableCtor, TextReplacements};

pub struct ConversationalChainInputCtor<I>(PhantomData<I>)
where
    I: InputVariableCtor;

impl<I> InputVariableCtor for ConversationalChainInputCtor<I>
where
    I: InputVariableCtor,
{
    type InputVariable<'a> = ConversationalChainInput<'a, I::InputVariable<'a>>;
}

pub struct ConversationalChainInput<'a, I = DefaultChainInput<'a>>
where
    I: InputVariable,
{
    pub inner: &'a I,
    pub chat_history: Option<Cow<'a, str>>,
}

impl<'a, I> ConversationalChainInput<'a, I>
where
    I: InputVariable,
{
    pub fn new(input: &'a I) -> Self {
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

impl<'a, I> InputVariable for ConversationalChainInput<'a, I>
where
    I: InputVariable,
{
    fn text_replacements(&self) -> TextReplacements {
        let mut replacements = self.inner.text_replacements();
        replacements.insert("history", self.chat_history.as_deref().unwrap_or("").into());
        replacements
    }
}
