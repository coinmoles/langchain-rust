use std::{borrow::Cow, collections::HashMap};

use crate::schemas::Message;

pub type TextReplacements<'a> = HashMap<&'a str, Cow<'a, str>>;
pub type PlaceholderReplacements<'a> = HashMap<&'a str, Vec<Message>>;

pub trait InputVariableCtor: Send + Sync {
    type InputVariable<'a>: InputVariable + 'a;
}

pub trait InputVariable: Send + Sync {
    fn text_replacements(&self) -> TextReplacements;
    fn placeholder_replacements(&self) -> PlaceholderReplacements {
        HashMap::new()
    }
}

impl InputVariable for HashMap<String, String> {
    fn text_replacements(&self) -> TextReplacements {
        self.iter()
            .map(|(k, v)| (k.as_str(), v.as_str().into()))
            .collect()
    }
}

impl InputVariable for HashMap<&str, &str> {
    fn text_replacements(&self) -> TextReplacements {
        self.iter().map(|(&k, &v)| (k, v.into())).collect()
    }
}

pub struct DefaultChainInputCtor;
impl InputVariableCtor for DefaultChainInputCtor {
    type InputVariable<'a> = DefaultChainInput<'a>;
}

pub struct DefaultChainInput<'a> {
    input: &'a str,
}

impl InputVariable for DefaultChainInput<'_> {
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
}

impl std::fmt::Display for DefaultChainInput<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
