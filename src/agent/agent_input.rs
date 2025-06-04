use std::borrow::Cow;

use crate::schemas::{
    ChainInput, ChainInputCtor, Message, PlaceholderReplacements, TextReplacements,
};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct AgentInputCtor<I>(std::marker::PhantomData<I>);
impl<I> ChainInputCtor for AgentInputCtor<I>
where
    I: ChainInputCtor,
{
    type Target<'a> = AgentInput<'a, I::Target<'a>>;
}

#[derive(Debug, Clone)]
pub struct AgentInput<'a, I>
where
    I: ChainInput,
{
    pub inner: Cow<'a, I>,
    pub agent_scratchpad: Option<Vec<Message>>,
    pub chat_history: Option<Vec<Message>>,
    pub ultimatum: bool,
}

impl<'a, I> AgentInput<'a, I>
where
    I: ChainInput,
{
    pub fn new(input: impl Into<Cow<'a, I>>) -> Self {
        Self {
            inner: input.into(),
            agent_scratchpad: None,
            chat_history: None,
            ultimatum: false,
        }
    }

    pub fn set_agent_scratchpad(&mut self, scratchpad: Vec<Message>) {
        self.agent_scratchpad = Some(scratchpad);
    }

    pub fn set_chat_history(&mut self, chat_history: Vec<Message>) {
        self.chat_history = Some(chat_history);
    }

    pub fn enable_ultimatum(&mut self) {
        self.ultimatum = true;
    }
}

impl<I> ChainInput for AgentInput<'_, I>
where
    I: ChainInput,
{
    fn text_replacements(&self) -> TextReplacements {
        self.inner.text_replacements()
    }

    fn placeholder_replacements(&self) -> PlaceholderReplacements {
        self.inner
            .placeholder_replacements()
            .into_iter()
            .chain([
                (
                    "agent_scratchpad",
                    self.agent_scratchpad.as_ref().unwrap().clone(),
                ),
                ("chat_history", self.chat_history.as_ref().unwrap().clone()),
                (
                    "ultimatum",
                    self.ultimatum
                        .then_some(vec![
                            Message::new_ai_message(""),
                            Message::new_human_message(FORCE_FINAL_ANSWER),
                        ])
                        .unwrap_or_default(),
                ),
            ])
            .collect()
    }
}
