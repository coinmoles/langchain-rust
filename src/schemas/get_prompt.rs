use crate::{schemas::Prompt, template::TemplateError};

pub trait GetPrompt<I> {
    fn get_prompt(&self, input: I) -> Result<Prompt, TemplateError>;
}
