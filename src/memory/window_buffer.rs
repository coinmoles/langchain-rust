use std::sync::Arc;

use tokio::sync::RwLock;

use super::Memory;
use crate::schemas::Message;

/// A window buffer memory implementation that retains only the most recent messages up to a
/// specified window size.
pub struct WindowBufferMemory {
    /// The maximum number of messages to retain in the memory.
    window_size: usize,
    /// The messages stored in the memory.
    messages: Vec<Message>,
}

impl WindowBufferMemory {
    /// Constructs a new [`WindowBufferMemory`] with the specified window size.
    pub fn new(window_size: usize) -> Self {
        Self {
            messages: Vec::new(),
            window_size,
        }
    }
}

impl Default for WindowBufferMemory {
    fn default() -> Self {
        Self::new(10)
    }
}

impl From<WindowBufferMemory> for Arc<dyn Memory> {
    fn from(val: WindowBufferMemory) -> Self {
        Arc::new(val)
    }
}

impl From<WindowBufferMemory> for Arc<RwLock<dyn Memory>> {
    fn from(val: WindowBufferMemory) -> Self {
        Arc::new(RwLock::new(val))
    }
}

impl Memory for WindowBufferMemory {
    fn messages(&self) -> Vec<Message> {
        self.messages.clone()
    }

    fn add_message(&mut self, message: Message) {
        if self.messages.len() >= self.window_size {
            self.messages.remove(0);
        }
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
