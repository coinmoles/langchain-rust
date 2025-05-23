use std::fmt;

use super::messages::Message;

#[derive(Debug, Clone)]
pub struct Prompt {
    messages: Vec<Message>,
}
impl Prompt {
    pub fn new(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    pub fn from_string(text: &str) -> Self {
        let message = Message::new_human_message(text);
        Self {
            messages: vec![message],
        }
    }

    pub fn to_messages(self) -> Vec<Message> {
        self.messages
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for message in &self.messages {
            writeln!(f, "{message}")?;
        }
        Ok(())
    }
}
