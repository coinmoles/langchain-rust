use crate::{schemas::InputVariable, schemas::Message};

#[derive(Debug, Clone)]
pub struct AgentInput<'a, I>
where
    I: InputVariable,
{
    pub inner: &'a I,
    pub agent_scratchpad: Option<Vec<Message>>,
    pub chat_history: Option<Vec<Message>>,
    pub ultimatum: bool,
}

impl<'a, I> AgentInput<'a, I>
where
    I: InputVariable,
{
    pub fn new(input: &'a I) -> Self {
        Self {
            inner: input,
            agent_scratchpad: None,
            chat_history: None,
            ultimatum: false,
        }
    }

    pub fn with_agent_scratchpad(mut self, scratchpad: Vec<Message>) -> Self {
        self.agent_scratchpad = Some(scratchpad);
        self
    }

    pub fn with_chat_history(mut self, chat_history: Vec<Message>) -> Self {
        self.chat_history = Some(chat_history);
        self
    }

    pub fn enable_ultimatum(mut self) -> Self {
        self.ultimatum = true;
        self
    }
}
