use std::collections::HashSet;

use crate::schemas::{InputVariables, Message, MessageType};
use crate::template::TemplateError;

#[derive(Clone)]
pub enum TemplateFormat {
    FString,
    Jinja2,
}

#[derive(Clone)]
pub struct MessageTemplate {
    message_type: MessageType,
    template: String,
    variables: HashSet<String>,
    format: TemplateFormat,
}

impl MessageTemplate {
    pub fn new(
        message_type: MessageType,
        template: impl Into<String>,
        variables: HashSet<String>,
        format: TemplateFormat,
    ) -> Self {
        Self {
            message_type,
            template: template.into(),
            variables,
            format,
        }
    }

    pub fn from_fstring(message_type: MessageType, content: impl Into<String>) -> Self {
        let content = content.into();

        let re = regex::Regex::new(r"\{(\w+)\}").unwrap();
        let variables = re
            .captures_iter(&content)
            .map(|cap| cap[1].to_string())
            .collect();

        Self::new(message_type, content, variables, TemplateFormat::FString)
    }

    pub fn from_jinja2(message_type: MessageType, content: impl Into<String>) -> Self {
        let content = content.into();

        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let variables = re
            .captures_iter(&content)
            .map(|cap| cap[1].to_string())
            .collect();

        Self::new(message_type, content, variables, TemplateFormat::Jinja2)
    }

    pub fn format(&self, input_variables: &InputVariables) -> Result<Message, TemplateError> {
        let mut content = self.template.clone();

        // check if all variables are in the input variables
        for key in &self.variables {
            if !input_variables.contains_text_key(key.as_str()) {
                return Err(TemplateError::MissingVariable(key.clone()));
            }
        }

        for (key, value) in input_variables.iter_test_replacements() {
            let key = match self.format {
                TemplateFormat::FString => format!("{{{key}}}"),
                TemplateFormat::Jinja2 => format!("{{{{{key}}}}}"),
            };
            content = content.replace(&key, value);
        }

        Ok(Message::new(self.message_type.clone(), content))
    }

    /// Returns a list of required input variable names for the template.
    pub fn variables(&self) -> HashSet<String> {
        self.variables.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::text_replacements;

    use super::*;

    #[test]
    fn test_fstring_template() {
        let template =
            MessageTemplate::from_fstring(MessageType::AIMessage, "Hello {name}, how are you?");

        let input_variables = text_replacements! {
            "name" => "Alice"
        }
        .into();

        let message = template.format(&input_variables).unwrap();
        assert_eq!(message.content, "Hello Alice, how are you?");
    }

    #[test]
    fn test_jinja2_template() {
        let template =
            MessageTemplate::from_jinja2(MessageType::AIMessage, "Hello {{name}}, how are you?");

        let input_variables = text_replacements! {
            "name" => "Alice"
        }
        .into();

        let message = template.format(&input_variables).unwrap();
        assert_eq!(message.content, "Hello Alice, how are you?");
    }

    #[test]
    fn test_jinja2_template_duplicate() {
        let template = MessageTemplate::from_jinja2(
            MessageType::AIMessage,
            "Hello {{name}}, how are you? Nice to meet you {{name}}!",
        );

        let input_variables = text_replacements! {
            "name" => "Alice"
        }
        .into();

        let message = template.format(&input_variables).unwrap();
        assert_eq!(
            message.content,
            "Hello Alice, how are you? Nice to meet you Alice!"
        );
    }
}
