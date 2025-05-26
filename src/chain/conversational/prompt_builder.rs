use crate::{
    chain::conversational::DEFAULT_INPUT_VARIABLE, schemas::TextReplacements, text_replacements,
};

///This is only useful when you dont modify the original prompt
pub struct ConversationalChainPromptBuilder<'a> {
    input: &'a str,
}

impl<'a> ConversationalChainPromptBuilder<'a> {
    pub fn new() -> Self {
        Self { input: "" }
    }

    pub fn input(mut self, input: &'a (impl AsRef<str> + ?Sized)) -> Self {
        self.input = input.as_ref();
        self
    }

    pub fn build(self) -> TextReplacements {
        text_replacements! {
            DEFAULT_INPUT_VARIABLE => self.input,
        }
    }
}

impl Default for ConversationalChainPromptBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
