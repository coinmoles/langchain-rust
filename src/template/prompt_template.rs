use std::collections::HashSet;

use super::MessageTemplate;
use crate::schemas::{InputVariable, Message, Prompt};
use crate::template::TemplateError;

#[derive(Clone)]
pub enum MessageOrTemplate {
    Message(Message),
    Template(MessageTemplate),
    Placeholder(String),
}

impl From<Message> for MessageOrTemplate {
    fn from(message: Message) -> Self {
        MessageOrTemplate::Message(message)
    }
}

impl From<MessageTemplate> for MessageOrTemplate {
    fn from(template: MessageTemplate) -> Self {
        MessageOrTemplate::Template(template)
    }
}

pub struct PromptTemplate {
    pub(crate) messages: Vec<MessageOrTemplate>,
}

impl PromptTemplate {
    pub fn new(messages: impl IntoIterator<Item = MessageOrTemplate>) -> Self {
        Self {
            messages: messages.into_iter().collect(),
        }
    }

    /// Insert variables into a prompt template to create a full-fletched prompt.
    ///
    /// replace_placeholder() must be called before format().
    pub fn format<'a>(&self, input: &impl InputVariable) -> Result<Prompt, TemplateError> {
        let text_replacements = input.text_replacements();
        let placeholder_replacements = input.placeholder_replacements();

        let messages = self
            .messages
            .iter()
            .flat_map(|m| -> Result<Vec<Message>, TemplateError> {
                match m {
                    MessageOrTemplate::Message(m) => Ok(vec![m.clone()]),
                    MessageOrTemplate::Template(t) => Ok(vec![t.format(&text_replacements)?]),
                    MessageOrTemplate::Placeholder(p) => {
                        match placeholder_replacements.get(p.as_str()) {
                            Some(messages) => Ok(messages.clone()),
                            None => Ok(vec![]),
                        }
                    }
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        Ok(Prompt::new(messages))
    }

    /// Returns a list of required input variable names for the template.
    pub fn variables(&self) -> HashSet<&str> {
        self.messages
            .iter()
            .filter_map(|m| match m {
                MessageOrTemplate::Template(t) => Some(t.variables()),
                _ => None,
            })
            .flatten()
            .collect()
    }

    pub fn placeholders(&self) -> HashSet<String> {
        let placeholders = self
            .messages
            .iter()
            .filter_map(|m| match m {
                MessageOrTemplate::Placeholder(p) => Some(p.clone()),
                _ => None,
            })
            .collect();

        placeholders
    }
}

impl From<MessageTemplate> for PromptTemplate {
    fn from(template: MessageTemplate) -> Self {
        Self::new(vec![MessageOrTemplate::Template(template)])
    }
}

#[macro_export]
macro_rules! prompt_template {
    ($($x:expr),*) => {
        $crate::template::PromptTemplate::new(vec![$($x.into()),*])
    };
}
