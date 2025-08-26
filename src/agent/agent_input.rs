use crate::{
    chain::{ChainInput, Ctor},
    schemas::Message,
};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

/// The input passed to an agent LLM.
///
/// Contains generic inner input for user provided fields and agent-specific fields for agent execution.
#[derive(Debug, Clone, ChainInput, Ctor)]
pub struct AgentInput<I: ChainInput> {
    /// The primary inner input to the agent, contains user-provided fields.
    #[langchain(into = "inner")]
    pub inner: I,

    /// Scratchpad for intermediate reasoning steps and tool calls.
    #[langchain(into = "placeholder")]
    pub agent_scratchpad: Option<Vec<Message>>,

    /// Previous chat history with the agent prior to this interaction.
    #[langchain(into = "placeholder")]
    pub chat_history: Option<Vec<Message>>,

    /// A special message used to signal that the agent must finalize its answer.
    #[langchain(into = "placeholder")]
    pub ultimatum: Option<Vec<Message>>,

    /// Extra key-value pairs injected during [`Strategy::prepare_input`].
    ///
    /// Only available when the `extra-keys` feature is enabled.
    #[cfg(feature = "extra-keys")]
    #[langchain(into = "inner")]
    pub extra_keys: std::collections::HashMap<String, String>,
}

impl<I: ChainInput> AgentInput<I> {
    /// Constructs a new [`AgentInput`]
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

    /// Sets the `agent_scratchpad` value.
    pub fn set_agent_scratchpad(&mut self, scratchpad: Vec<Message>) {
        self.agent_scratchpad = Some(scratchpad);
    }

    /// Sets the `chat_history` value.
    pub fn set_chat_history(&mut self, chat_history: Vec<Message>) {
        self.chat_history = Some(chat_history);
    }

    /// Enables ultimatum which forces LLM to provide a final answer on the next step.
    pub fn enable_ultimatum(&mut self) {
        self.ultimatum = Some(vec![
            Message::new_ai_message(""),
            Message::new_human_message(FORCE_FINAL_ANSWER),
        ]);
    }

    /// Adds extra agent-specific fields to override agent executor behavior.
    #[cfg(feature = "extra-keys")]
    pub fn add_extra_key(&mut self, key: String, value: String) {
        self.extra_keys.insert(key, value);
    }
}
