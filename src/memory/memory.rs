use crate::schemas::{Message, ToolCall};

/// Long-term memory to store and retrieve messages.
///
/// The memory objects are most notably used by [`AgentExecutor`](crate::agent::AgentExecutor) to
/// store chat history across executions.
pub trait Memory: Send + Sync {
    /// Returns all messages stored in memory.
    fn messages(&self) -> Vec<Message>;

    /// Adds a message to the memory.
    fn add_message(&mut self, message: Message);

    /// Adds multiple messages to the memory.
    fn add_messages(&mut self, messages: Vec<Message>) {
        for message in messages {
            self.add_message(message);
        }
    }

    /// Clears all messages from the memory.
    fn clear(&mut self);

    /// Converts the memory contents to a string representation.
    fn to_string(&self) -> String;

    /// Adds a human message to the memory.
    fn add_human_message(&mut self, content: String) {
        self.add_message(Message::new_human_message(content))
    }

    /// Adds an AI message to the memory.
    fn add_ai_message(&mut self, content: String) {
        self.add_message(Message::new_ai_message(content))
    }

    /// Adds a tool call message to the memory.
    fn add_tool_call_message(&mut self, thought: Option<String>, tool_calls: Vec<ToolCall>) {
        self.add_message(Message::new_tool_call_message(thought, tool_calls))
    }

    /// Adds a tool message to the memory.
    fn add_tool_message(&mut self, id: Option<String>, content: String) {
        self.add_message(Message::new_tool_message(id, content))
    }
}

impl<M> From<M> for Box<dyn Memory>
where
    M: Memory + 'static,
{
    fn from(memory: M) -> Self {
        Box::new(memory)
    }
}
