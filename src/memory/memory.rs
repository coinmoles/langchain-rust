use crate::schemas::{Message, ToolCall};

pub trait Memory: Send + Sync {
    fn messages(&self) -> Vec<Message>;

    fn add_message(&mut self, message: Message);

    fn add_messages(&mut self, messages: Vec<Message>) {
        for message in messages {
            self.add_message(message);
        }
    }

    fn clear(&mut self);

    fn to_string(&self) -> String;

    fn update(
        &mut self,
        initial_messages: Vec<Message>,
        scratchpad: Vec<Message>,
        final_answer: String,
    ) {
        self.add_messages(initial_messages);
        self.add_messages(scratchpad);
        self.add_ai_message(final_answer);
    }

    fn add_human_message(&mut self, content: String) {
        self.add_message(Message::new_human_message(content))
    }

    fn add_ai_message(&mut self, content: String) {
        self.add_message(Message::new_ai_message(content))
    }

    fn add_tool_call_message(&mut self, thought: Option<String>, tool_calls: Vec<ToolCall>) {
        self.add_message(Message::new_tool_call_message(thought, tool_calls))
    }

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
