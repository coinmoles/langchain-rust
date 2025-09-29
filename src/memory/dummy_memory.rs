use std::sync::Arc;

use tokio::sync::RwLock;

use super::Memory;
use crate::schemas::Message;

/// A dummy memory implementation that does not store any messages.
#[derive(Debug, Clone)]
pub struct DummyMemory;

impl DummyMemory {
    /// Constructs a new `DummyMemory`.
    pub fn new() -> Self {
        Self
    }
}

impl Default for DummyMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DummyMemory> for Arc<dyn Memory> {
    fn from(val: DummyMemory) -> Self {
        Arc::new(val)
    }
}

impl From<DummyMemory> for Arc<RwLock<dyn Memory>> {
    fn from(val: DummyMemory) -> Self {
        Arc::new(RwLock::new(val))
    }
}

impl Memory for DummyMemory {
    fn messages(&self) -> Vec<Message> {
        vec![]
    }

    fn add_message(&mut self, _message: Message) {}

    fn clear(&mut self) {}

    fn to_string(&self) -> String {
        "".to_string()
    }
}
