pub use macros::ChainInput;
use std::{borrow::Cow, collections::HashMap};

use crate::{chain::Ctor, schemas::Message};

pub type TextReplacements<'a> = HashMap<&'a str, Cow<'a, str>>;
pub type PlaceholderReplacements<'a> = HashMap<&'a str, Cow<'a, [Message]>>;

pub trait ChainInput: Send + Sync {
    fn text_replacements(&self) -> TextReplacements<'_>;
    fn placeholder_replacements(&self) -> PlaceholderReplacements<'_> {
        HashMap::new()
    }
}

impl ChainInput for () {
    fn text_replacements(&self) -> TextReplacements<'_> {
        HashMap::new()
    }
}

impl ChainInput for HashMap<String, String> {
    fn text_replacements(&self) -> TextReplacements<'_> {
        self.iter()
            .map(|(k, v)| (k.as_str(), v.as_str().into()))
            .collect()
    }
}

impl ChainInput for HashMap<&str, &str> {
    fn text_replacements(&self) -> TextReplacements<'_> {
        self.iter().map(|(&k, &v)| (k, v.into())).collect()
    }
}

#[derive(Clone, Default, ChainInput, Ctor)]
pub struct DefaultChainInput<'a> {
    #[langchain(into = "text")]
    input: &'a str,
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

impl<'a> From<&'a str> for DefaultChainInput<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}

impl std::fmt::Display for DefaultChainInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
