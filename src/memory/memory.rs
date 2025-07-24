use crate::{
    agent::AgentStep,
    schemas::{Message, ToolCall},
};

pub trait Memory: Send + Sync {
    fn messages(&self) -> Vec<Message>;

    fn add_message(&mut self, message: Message);

    fn clear(&mut self);

    fn to_string(&self) -> String;

    fn update(&mut self, human_message: String, steps: Vec<AgentStep>, final_answer: String) {
        self.add_human_message(human_message);
        for step in steps {
            let tool_call_id = step.tool_call.id.clone();
            self.add_tool_call_message(vec![step.tool_call]);
            self.add_tool_message(Some(tool_call_id), step.output.data.to_string());
        }
        self.add_ai_message(final_answer);
    }

    fn add_human_message(&mut self, content: String) {
        self.add_message(Message::new_human_message(content))
    }

    fn add_ai_message(&mut self, content: String) {
        self.add_message(Message::new_ai_message(content))
    }

    fn add_tool_call_message(&mut self, tool_calls: Vec<ToolCall>) {
        self.add_message(Message::new_tool_call_message(tool_calls))
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
