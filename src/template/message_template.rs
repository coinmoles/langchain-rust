use std::collections::HashSet;

use crate::chain::TextReplacements;
use crate::schemas::{Message, Role};
use crate::template::TemplateError;

/// The message template format.
#[derive(Debug, Clone)]
pub enum TemplateFormat {
    /// Python-style f-string formatting.
    FString,
    /// Jinja2-style formatting.
    Jinja2,
}

/// A message template that can be formatted with input variables to produce a [`Message`].
#[derive(Debug, Clone)]
pub struct MessageTemplate {
    /// The message role. e.g. system, ai, human, tool
    message_type: Role,
    /// The template string with placeholders for variables.
    template: String,
    /// The set of variable names required by the template.
    variables: HashSet<String>,
    /// The format of the template.
    format: TemplateFormat,
}

impl MessageTemplate {
    /// Constructs a new [`MessageTemplate`].
    pub fn new(
        message_type: Role,
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

    /// Constructs a new [`MessageTemplate`] from a Python-style f-string template.
    pub fn from_fstring(message_type: Role, content: impl Into<String>) -> Self {
        let content = content.into();

        let re = regex::Regex::new(r"\{(\w+)\}").unwrap();
        let variables = re
            .captures_iter(&content)
            .map(|cap| cap[1].to_string())
            .collect();

        Self::new(message_type, content, variables, TemplateFormat::FString)
    }

    /// Constructs a new [`MessageTemplate`] from a Jinja2-style template.
    pub fn from_jinja2(message_type: Role, content: impl Into<String>) -> Self {
        let content = content.into();

        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let variables = re
            .captures_iter(&content)
            .map(|cap| cap[1].to_string())
            .collect();

        Self::new(message_type, content, variables, TemplateFormat::Jinja2)
    }

    /// Formats the template with the provided input variables, returning a [`Message`].
    ///
    /// # Errors
    /// Returns a [`TemplateError::MissingVariable`] if any required variables are missing from the
    /// input.
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

    /// Returns a set of variable names required by the template.
    pub fn variables(&self) -> HashSet<&str> {
        self.variables.iter().map(String::as_str).collect()
    }

    /// Validates that all required variables are present in the input.
    ///
    /// # Errors
    /// Returns a [`TemplateError::MissingVariable`] if any required variables are missing from the
    /// input.
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
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        "Hello {name}, how are you?",
        TextReplacements::from([("name", "Alice".into())]),
        "Hello Alice, how are you?"
    )]
    fn test_template_fstring(
        #[case] template: &str,
        #[case] input: TextReplacements,
        #[case] expected: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::from_fstring(Role::Ai, template);
        let message = template.format(&input)?;
        assert_eq!(message.content, expected);
        Ok(())
    }

    #[rstest]
    #[case::simple(
        "Hello {{name}}, how are you?",
        TextReplacements::from([("name", "Alice".into())]),
        "Hello Alice, how are you?"
    )]
    #[case::multiple(
        "Hello {{name}}, your order {{order_id}} is confirmed.",
        TextReplacements::from([("name", "Alice".into()), ("order_id", "12345".into())]),
        "Hello Alice, your order 12345 is confirmed."
    )]
    #[case::duplicate(
        "Your order {{id}} is confirmed. Order {{id}} will arrive soon.",
        TextReplacements::from([("id", "12345".into())]),
        "Your order 12345 is confirmed. Order 12345 will arrive soon."
    )]
    #[case::empty(
        "Braces with no variable: {{ }}",
        TextReplacements::new(),
        "Braces with no variable: {{ }}"
    )]
    #[case::weird(
        "Edge {{name}}}} test",
        TextReplacements::from([("name", "Charlie".into())]),
        "Edge Charlie}} test"
    )]

    fn test_template_jinja2(
        #[case] template: &str,
        #[case] input: TextReplacements,
        #[case] expected: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let template = MessageTemplate::from_jinja2(Role::Ai, template);
        let message = template.format(&input)?;
        assert_eq!(message.content, expected);
        Ok(())
    }

    #[rstest]
    #[case(
        "Hello {{name}}",
        TextReplacements::new() // missing "name"
    )]
    #[case(
        "Hello {{first}} {{last}}",
        TextReplacements::from([("first", "Alice".into())]) // missing "last"
    )]
    fn test_template_jinja2_missing_vars(#[case] template: &str, #[case] input: TextReplacements) {
        let template = MessageTemplate::from_jinja2(Role::Ai, template);
        let result = template.format(&input);

        let Err(TemplateError::MissingVariable(_)) = result else {
            panic!("Expected TemplateError::MissingVariable");
        };
    }
}
