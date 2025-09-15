use std::collections::HashMap;

use crate::{
    agent::Agent,
    chain::{InputCtor, LLMChain, OutputCtor},
    llm::LLM,
    prompt_template,
    schemas::MessageType,
    template::{MessageOrTemplate, MessageTemplate},
    tools::Tool,
    utils::helper::normalize_tool_name,
};

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"Assistant is designed to be able to assist with a wide range of tasks, from answering simple questions to providing in-depth explanations and discussions on a wide range of topics. As a language model, Assistant is able to generate human-like text based on the input it receives, allowing it to engage in natural-sounding conversations and provide responses that are coherent and relevant to the topic at hand.

Assistant is constantly learning and improving, and its capabilities are constantly evolving. It is able to process and understand large amounts of text, and can use this knowledge to provide accurate and informative responses to a wide range of questions. Additionally, Assistant is able to generate its own text based on the input it receives, allowing it to engage in discussions and provide explanations and descriptions on a wide range of topics.

Overall, Assistant is a powerful system that can help with a wide range of tasks and provide valuable insights and information on a wide range of topics. Whether you need help with a specific question or just want to have a conversation about a particular topic, Assistant is here to assist."#;

pub const DEFAULT_INITIAL_PROMPT: &str = r#"{{input}}"#;

/// A builder for constructing an [`OpenAiToolAgent`].
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
pub struct AgentBuilder<'a, 'b, I: InputCtor, O: OutputCtor> {
    /// The tools to be used by the agent.
    tools: Option<Vec<Tool>>,
    // /// The toolboxes containing additional tools for the agent.
    // toolboxes: Option<Vec<Box<dyn Toolbox>>>,
    /// The system prompt to be used by the agent.
    system_prompt: Option<&'a str>,
    /// The initial user prompt to be used by the agent.
    initial_prompt: Option<&'b str>,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> AgentBuilder<'a, 'b, I, O> {
    /// Constructs a new [`AgentBuilder`].
    ///
    /// This is the same as calling [`OpenAiToolAgent::builder()`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: None,
            system_prompt: None,
            initial_prompt: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Adds tools.
    pub fn tools(mut self, tools: impl IntoIterator<Item = Tool>) -> Self {
        self.tools = Some(tools.into_iter().collect());
        self
    }

    // /// Adds toolboxes.
    // pub fn toolboxes(mut self, toolboxes: Vec<Box<dyn Toolbox>>) -> Self {
    //     self.toolboxes = Some(toolboxes);
    //     self
    // }

    /// Sets the system prompt.
    pub fn system_prompt(mut self, system_prompt: &'a str) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    /// Sets the initial prompt.
    pub fn initial_prompt(mut self, initial_prompt: &'b str) -> Self {
        self.initial_prompt = Some(initial_prompt);
        self
    }

    /// Returns a [`OpenAiToolAgent`] that uses this [`AgentBuilder`] configuration.
    pub fn build<L: LLM + 'static>(self, llm: L) -> Agent<I, O> {
        let system_prompt = self.system_prompt.unwrap_or(DEFAULT_SYSTEM_PROMPT);
        let initial_prompt = self.initial_prompt.unwrap_or(DEFAULT_INITIAL_PROMPT);

        // let toolboxes = self.toolboxes.unwrap_or_default();
        let tools = self
            .tools
            .unwrap_or_default()
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect::<HashMap<_, _>>();

        let prompt = prompt_template![
            MessageTemplate::from_jinja2(MessageType::System, system_prompt),
            MessageOrTemplate::Placeholder("chat_history".into()),
            MessageTemplate::from_jinja2(MessageType::Human, initial_prompt),
            MessageOrTemplate::Placeholder("agent_scratchpad".into()),
            MessageOrTemplate::Placeholder("ultimatum".into())
        ];
        let llm_chain = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .build()
            .unwrap_or_else(|_| unreachable!("All necessary fields are provided"));

        Agent {
            id: uuid::Uuid::new_v4().to_string(),
            llm_chain,
            tools,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, 'b, I: InputCtor, O: OutputCtor> Default for AgentBuilder<'a, 'b, I, O> {
    fn default() -> Self {
        Self::new()
    }
}
