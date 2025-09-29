use std::sync::Arc;

use tokio::sync::RwLock;

use super::Memory;
use crate::schemas::Message;

/// A simple in-memory memory implementation that saves all messsages as is.
#[derive(Debug, Clone)]
pub struct SimpleMemory {
    /// The messages stored in the memory.
    messages: Vec<Message>,
}

impl SimpleMemory {
    /// Constructs a new [`SimpleMemory`].
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl From<SimpleMemory> for Arc<dyn Memory> {
    fn from(val: SimpleMemory) -> Self {
        Arc::new(val)
    }
}

impl From<SimpleMemory> for Arc<RwLock<dyn Memory>> {
    fn from(val: SimpleMemory) -> Self {
        Arc::new(RwLock::new(val))
    }
}

impl Memory for SimpleMemory {
    fn messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    fn add_messages(&mut self, messages: Vec<Message>) {
        self.messages.extend(messages);
    }

    fn clear(&mut self) {
        self.messages.clear();
    }

    fn to_string(&self) -> String {
        self.messages
            .iter()
            .map(|msg| msg.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
