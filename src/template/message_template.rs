use std::collections::HashSet;

use crate::{
    chain::TextReplacements,
    schemas::{Message, MessageType},
    template::TemplateError,
};

#[derive(Debug, Clone)]
pub enum TemplateFormat {
    FString,
    Jinja2,
}

#[derive(Debug, Clone)]
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

    pub fn format(&self, input: &TextReplacements) -> Result<Message, TemplateError> {
        self.validate_input(input)?;

        let mut content = self.template.clone();

        for (key, value) in input {
            let key = match self.format {
                TemplateFormat::FString => format!("{{{key}}}"),
                TemplateFormat::Jinja2 => format!("{{{{{key}}}}}"),
            };
            content = content.replace(&key, value);
        }

        Ok(Message::new(self.message_type.clone(), content))
    }

    /// Returns a list of required input variable names for the template.
    pub fn variables(&self) -> HashSet<&str> {
        self.variables.iter().map(String::as_str).collect()
    }

    pub fn validate_input(&self, input: &TextReplacements) -> Result<(), TemplateError> {
        let missing_variables = self
            .variables()
            .difference(&input.keys().cloned().collect())
            .cloned()
            .collect::<Vec<_>>();

        if !missing_variables.is_empty() {
            return Err(TemplateError::MissingVariable(missing_variables.join(", ")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_fstring_template() {
        let template = MessageTemplate::from_fstring(MessageType::Ai, "Hello {name}, how are you?");

        let input = HashMap::from([("name", "Alice".into())]);

        let message = template.format(&input).unwrap();
        assert_eq!(message.content, "Hello Alice, how are you?");
    }

    #[test]
    fn test_jinja2_template() {
        let template =
            MessageTemplate::from_jinja2(MessageType::Ai, "Hello {{name}}, how are you?");

        let input_variables = HashMap::from([("name", "Alice".into())]);

        let message = template.format(&input_variables).unwrap();
        assert_eq!(message.content, "Hello Alice, how are you?");
    }

    #[test]
    fn test_jinja2_template_duplicate() {
        let template = MessageTemplate::from_jinja2(
            MessageType::Ai,
            "Hello {{name}}, how are you? Nice to meet you {{name}}!",
        );

        let input_variables = HashMap::from([("name", "Alice".into())]);

        let message = template.format(&input_variables).unwrap();
        assert_eq!(
            message.content,
            "Hello Alice, how are you? Nice to meet you Alice!"
        );
    }
}
