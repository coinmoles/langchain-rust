use crate::{
    schemas::{ChainInput, Prompt},
    template::TemplateError,
};

pub trait GetPrompt<I: ChainInput> {
    fn get_prompt(&self, input: &I) -> Result<Prompt, TemplateError>;
}
