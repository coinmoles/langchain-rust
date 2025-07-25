use crate::{
    chain::{ChainInput, Ctor},
    schemas::Message,
};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

#[derive(Debug, Clone, ChainInput, Ctor)]
pub struct AgentInput<I: ChainInput> {
    #[langchain(into = "inner")]
    pub inner: I,
    #[langchain(into = "placeholder")]
    pub agent_scratchpad: Option<Vec<Message>>,
    #[langchain(into = "placeholder")]
    pub chat_history: Option<Vec<Message>>,
    #[langchain(into = "placeholder")]
    pub ultimatum: Option<Vec<Message>>,
    #[cfg(feature = "extra-keys")]
    #[langchain(into = "inner")]
    pub extra_keys: std::collections::HashMap<String, String>,
}

impl<I: ChainInput> AgentInput<I> {
    pub fn new(input: I) -> Self {
        Self {
            inner: input,
            agent_scratchpad: None,
            chat_history: None,
            ultimatum: None,
            #[cfg(feature = "extra-keys")]
            extra_keys: std::collections::HashMap::new(),
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

    #[cfg(feature = "extra-keys")]
    pub fn add_extra_key(&mut self, key: String, value: String) {
        self.extra_keys.insert(key, value);
    }
}
