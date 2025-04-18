use std::sync::Arc;

use tokio::sync::RwLock;

use crate::schemas::messages::Message;

use super::Memory;

pub struct SimpleMemory {
    messages: Vec<Message>,
}

impl SimpleMemory {
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

    fn clear(&mut self) {
        self.messages.clear();
    }

    fn to_string(&self) -> String {
        self.messages()
            .iter()
            .map(|msg| msg.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}
