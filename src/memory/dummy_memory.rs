use std::sync::Arc;

use tokio::sync::Mutex;

use crate::schemas::messages::Message;

use super::Memory;

pub struct DummyMemory {}

impl DummyMemory {
    pub fn new() -> Self {
        Self {}
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

impl From<DummyMemory> for Arc<Mutex<dyn Memory>> {
    fn from(val: DummyMemory) -> Self {
        Arc::new(Mutex::new(val))
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
