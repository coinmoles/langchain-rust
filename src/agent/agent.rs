use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::agent::{AgentBuilder, AgentExecutor};
use crate::chain::{
    ChainOutput, DefaultChainInputCtor, GetPrompt, InputCtor, OutputCtor, StringCtor,
};
use crate::llm::LLM;
use crate::schemas::Prompt;
use crate::template::{PromptTemplate, TemplateError};
use crate::tools::{Tool, Toolbox};

/// An LLM agent that iteratively plan / execute tool actions until producing a valid final answer.
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type (defaults to
///   [`DefaultChainInputCtor`], which constructs [`ChainInput`](crate::chain::DefaultChainInput)).
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type (defaults to
///   [`StringCtor`], which constructs [`String`]).
pub struct Agent<'tool, I: InputCtor = DefaultChainInputCtor, O: OutputCtor = StringCtor> {
    /// A unique identifier for the agent. Used for logging.
    pub(super) id: String,
    /// The prompt template for the agent.
    pub(super) prompt: PromptTemplate,
    /// The LLM used by the agent.
    pub(super) llm: Box<dyn LLM>,
    /// A map of registered tool names to their implementations.
    pub(super) tools: HashMap<String, Tool<'tool>>,
    /// A list of toolboxes used to dynamically provide tools at runtime.
    pub(super) toolboxes: Vec<Arc<dyn Toolbox>>,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<'tool, I: InputCtor, O: OutputCtor> Agent<'tool, I, O> {
    /// Constructs a new [`Agent`].
    ///
    /// It is recommended to use [`Agent::builder()`] to create an agent instead of calling the
    /// constructor directly.
    pub fn new(
        id: String,
        prompt: PromptTemplate,
        llm: Box<dyn LLM>,
        tools: HashMap<String, Tool<'tool>>,
        toolboxes: Vec<Arc<dyn Toolbox>>,
    ) -> Self {
        Self {
            id,
            prompt,
            llm,
            tools,
            toolboxes,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Creates a [`AgentBuilder`] to configure an [`Agent`].
    ///
    /// This is the same as calling [`AgentBuilder::new()`].
    ///
    /// # Example:
    /// ```
    /// use async_openai::config::OpenAIConfig;
    /// use langchain_rust::agent::Agent;
    /// use langchain_rust::llm::{OpenAIChat, OpenAIModel};
    ///
    /// let llm: OpenAIChat<OpenAIConfig> =
    ///     OpenAIChat::builder().with_model(OpenAIModel::Gpt4o).build();
    ///
    /// let agent: Agent = Agent::builder()
    ///     .system_prompt("You are a helpful assistant.")
    ///     .initial_prompt("Help me find {{input}}.")
    ///     // .tools(vec![my_tool]) // You can add tools here
    ///     .build(llm);
    /// ```
    pub fn builder<'a, 'b>() -> AgentBuilder<'a, 'b, 'tool, I, O> {
        AgentBuilder::new()
    }

    /// Creates an [`AgentExecutor`] from this agent.
    ///
    /// # Example:
    /// ```
    /// use async_openai::config::OpenAIConfig;
    /// use langchain_rust::agent::Agent;
    /// use langchain_rust::llm::{OpenAIChat, OpenAIModel};
    ///
    /// let llm: OpenAIChat<OpenAIConfig> =
    ///     OpenAIChat::builder().with_model(OpenAIModel::Gpt4o).build();
    ///
    /// let agent: Agent = Agent::builder()
    ///     .system_prompt("You are a helpful assistant.")
    ///     .initial_prompt("Help me find {{input}}.")
    ///     // .tools(vec![my_tool]) // You can add tools here
    ///     .build(llm);
    /// ```
    pub fn executor(self) -> AgentExecutor<'tool, I, O>
    where
        for<'any> I::Target<'any>: Display,
        for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
    {
        AgentExecutor::from_agent(self)
    }

    /// Returns the unique identifier of the agent.
    pub fn id(&self) -> &str {
        self.id.as_str()
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<I::Target<'_>> for Agent<'_, I, O> {
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.prompt.format(&input)
    }
}
