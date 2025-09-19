use std::fmt;

use super::message::Message;

/// An LLM prompt consisting of a series of messages.
#[derive(Debug, Clone)]
pub struct Prompt {
    /// The messages in this prompt.
    messages: Vec<Message>,
}

impl Prompt {
    /// Constructs a new [`Prompt`] from a vector of [`Message`]s.
    pub fn new(messages: Vec<Message>) -> Self {
        Self { messages }
    }

    /// Constructs a new [`Prompt`] with a single human message.
    pub fn single(text: &str) -> Self {
        let message = Message::new_human_message(text);
        Self {
            messages: vec![message],
        }
    }

    /// Returns the messages in this prompt.
    pub fn to_messages(self) -> Vec<Message> {
        self.messages
    }
}

impl IntoIterator for Prompt {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.messages.into_iter()
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
