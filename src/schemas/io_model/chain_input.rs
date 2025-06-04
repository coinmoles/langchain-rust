use std::{borrow::Cow, collections::HashMap};

use crate::schemas::Message;

pub type TextReplacements<'a> = HashMap<&'a str, Cow<'a, str>>;
pub type PlaceholderReplacements<'a> = HashMap<&'a str, Vec<Message>>;

pub trait ChainInputCtor: Send + Sync {
    type Target<'a>: ChainInput + 'a;
}

pub trait ChainInput: Clone + Send + Sync {
    fn text_replacements(&self) -> TextReplacements;
    fn placeholder_replacements(&self) -> PlaceholderReplacements {
        HashMap::new()
    }
}

impl ChainInput for HashMap<String, String> {
    fn text_replacements(&self) -> TextReplacements {
        self.iter()
            .map(|(k, v)| (k.as_str(), v.as_str().into()))
            .collect()
    }
}

impl ChainInput for HashMap<&str, &str> {
    fn text_replacements(&self) -> TextReplacements {
        self.iter().map(|(&k, &v)| (k, v.into())).collect()
    }
}

pub struct DefaultChainInputCtor;
impl ChainInputCtor for DefaultChainInputCtor {
    type Target<'a> = DefaultChainInput<'a>;
}

#[derive(Clone, Default)]
pub struct DefaultChainInput<'a> {
    input: &'a str,
}

impl ChainInput for DefaultChainInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([("input", self.input.into())])
    }
}

impl<'a> DefaultChainInput<'a> {
    pub fn new(input: &'a (impl AsRef<str> + ?Sized)) -> Self {
        Self {
            input: input.as_ref(),
        }
    }

    pub fn input(mut self, input: &'a str) -> Self {
        self.input = input;
        self
    }
}

impl std::fmt::Display for DefaultChainInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
