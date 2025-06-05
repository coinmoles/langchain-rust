use std::borrow::Cow;

use crate::schemas::{ChainInput, ChainInputCtor, Message};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct AgentInputCtor<I: ChainInputCtor>(std::marker::PhantomData<I>);
impl<I: ChainInputCtor> ChainInputCtor for AgentInputCtor<I> {
    type Target<'a> = AgentInput<'a, I::Target<'a>>;
}

#[derive(Debug, Clone, ChainInput)]
pub struct AgentInput<'a, I: ChainInput> {
    #[chain_input(inner)]
    pub inner: Cow<'a, I>,
    #[chain_input(placeholder)]
    pub agent_scratchpad: Option<Vec<Message>>,
    #[chain_input(placeholder)]
    pub chat_history: Option<Vec<Message>>,
    #[chain_input(placeholder)]
    pub ultimatum: Option<Vec<Message>>,
}

impl<'a, I: ChainInput> AgentInput<'a, I> {
    pub fn new(input: impl Into<Cow<'a, I>>) -> Self {
        Self {
            inner: input.into(),
            agent_scratchpad: None,
            chat_history: None,
            ultimatum: None,
        }
    }

    pub fn set_agent_scratchpad(&mut self, scratchpad: Vec<Message>) {
        self.agent_scratchpad = Some(scratchpad);
    }

    pub fn set_chat_history(&mut self, chat_history: Vec<Message>) {
        self.chat_history = Some(chat_history);
    }

    pub fn enable_ultimatum(&mut self) {
        self.ultimatum = Some(vec![
            Message::new_ai_message(""),
            Message::new_human_message(FORCE_FINAL_ANSWER),
        ]);
    }
}
