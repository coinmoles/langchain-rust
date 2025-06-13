use crate::{
    chain::{ChainInput, Ctor, InputCtor},
    schemas::Message,
};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct AgentInputCtor<I: InputCtor>(std::marker::PhantomData<I>);
impl<I: InputCtor> Ctor for AgentInputCtor<I> {
    type Target<'a> = AgentInput<I::Target<'a>>;
}

#[derive(Debug, Clone, ChainInput)]
pub struct AgentInput<I: ChainInput> {
    #[langchain(into = "inner")]
    pub inner: I,
    #[langchain(into = "placeholder")]
    pub agent_scratchpad: Option<Vec<Message>>,
    #[langchain(into = "placeholder")]
    pub chat_history: Option<Vec<Message>>,
    #[langchain(into = "placeholder")]
    pub ultimatum: Option<Vec<Message>>,
}

impl<I: ChainInput> AgentInput<I> {
    pub fn new(input: I) -> Self {
        Self {
            inner: input,
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
