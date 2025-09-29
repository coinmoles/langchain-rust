use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::agent::{AgentBuilder, AgentExecutor, AgentInput, AgentInputCtor};
use crate::chain::{
    ChainOutput, DefaultChainInputCtor, GetPrompt, InputCtor, LLMChain, OutputCtor, StringCtor,
};
use crate::schemas::{LLMOutputCtor, Prompt};
use crate::template::TemplateError;
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
    /// The inner [`LLMChain`] used for prompt construction and LLM invocation.
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, LLMOutputCtor>,
    /// A map of registered tool names to their implementations.
    pub(super) tools: HashMap<String, Tool<'tool>>,
    /// A list of toolboxes used to dynamically provide tools at runtime.
    pub(super) toolboxes: Vec<Arc<dyn Toolbox>>,
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<'tool, I: InputCtor, O: OutputCtor> Agent<'tool, I, O> {
    /// Constructs a new `Agent` with the given LLM chain and tools.
    ///
    /// It is recommended to use [`Agent::builder()`] to create an agent instead of calling the
    /// constructor directly.
    ///
    /// # Arguments
    /// - `id`: A unique identifier for the agent.
    /// - `llm_chain`: The [`LLMChain`] to use for prompt construction and LLM invocation.
    /// - `tools`: A vector of [`Tool`]s that the agent can use.
    /// - `toolboxes`: A vector of [`Toolbox`]es that the agent can use to dynamically provide
    ///   tools.
    ///
    /// # Returns
    /// A new instance of `Agent`.
    pub fn new(
        id: String,
        llm_chain: LLMChain<AgentInputCtor<I>, LLMOutputCtor>,
        tools: HashMap<String, Tool<'tool>>,
        toolboxes: Vec<Arc<dyn Toolbox>>,
    ) -> Self {
        Self {
            id,
            llm_chain,
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

    /// Returns the prompt used by the agent.
    pub fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }
}
