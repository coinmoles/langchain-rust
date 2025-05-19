use std::collections::HashSet;

use super::MessageTemplate;
use crate::schemas::{InputVariables, Message, Prompt};
use crate::template::TemplateError;

#[derive(Clone)]
pub enum MessageOrTemplate {
    Message(Message),
    Template(MessageTemplate),
    Placeholder(String),
}

pub struct PromptTemplate {
    messages: Vec<MessageOrTemplate>,
}

impl PromptTemplate {
    pub fn new(messages: impl IntoIterator<Item = MessageOrTemplate>) -> Self {
        Self {
            messages: messages.into_iter().collect(),
        }
    }

    pub fn insert_message(&mut self, message: Message) {
        self.messages.push(MessageOrTemplate::Message(message));
    }

    pub fn insert_template(&mut self, template: MessageTemplate) {
        self.messages.push(MessageOrTemplate::Template(template));
    }

    pub fn insert_placeholder(&mut self, placeholder: String) {
        self.messages
            .push(MessageOrTemplate::Placeholder(placeholder));
    }

    /// Insert variables into a prompt template to create a full-fletched prompt.
    ///
    /// replace_placeholder() must be called before format().
    pub fn format(&self, input_variables: &InputVariables) -> Result<Prompt, TemplateError> {
        let messages = self
            .messages
            .iter()
            .flat_map(|m| -> Result<Vec<Message>, TemplateError> {
                match m {
                    MessageOrTemplate::Message(m) => Ok(vec![m.clone()]),
                    MessageOrTemplate::Template(t) => Ok(vec![t.format(input_variables)?]),
                    MessageOrTemplate::Placeholder(p) => {
                        match input_variables.get_placeholder_replacement(p) {
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
    pub fn variables(&self) -> HashSet<String> {
        let variables = self
            .messages
            .iter()
            .flat_map(|m| match m {
                MessageOrTemplate::Template(t) => t.variables(),
                _ => HashSet::new(),
            })
            .collect();

        variables
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

#[macro_export]
macro_rules! prompt_template {
    ($($x:expr),*) => {
        $crate::template::PromptTemplate::new(vec![$($x.into()),*])
    };
}
