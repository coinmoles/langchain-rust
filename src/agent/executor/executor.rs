use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    agent::{Agent, AgentInput, DefaultStrategy, ExecutionContext, Strategy},
    chain::{Chain, ChainError, ChainOutput, InputCtor, OutputCtor},
    memory::Memory,
    schemas::{GetPrompt, Prompt, WithUsage},
    template::TemplateError,
};

use super::ExecutorOptions;

/// A runtime executor for driving multi-step agent execution with memory, planning, and tool use.
///
/// This struct coordinates the full reasoning loop of an [`Agent`](crate::agent::Agent), handling prompt generation,
/// scratchpad construction, tool resolution, and optional memory integration. It provides a high-level interface
/// for running agents in a predictable, type-safe, and optionally stateful manner.
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
pub struct AgentExecutor<'agent, I: InputCtor, O: OutputCtor>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) agent: Box<dyn Agent<I, O> + 'agent>,
    pub(super) memory: Option<Arc<RwLock<dyn Memory>>>,
    pub(super) options: ExecutorOptions,
}

impl<'agent, I: InputCtor, O: OutputCtor> AgentExecutor<'agent, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    /// Constructs a new [`AgentExecutor`] from a struct that implements the trait [`Agent`].
    pub fn from_agent(agent: impl Agent<I, O> + 'agent) -> Self {
        Self {
            agent: Box::new(agent),
            memory: None,
            options: ExecutorOptions::default(),
        }
    }

    /// Sets the memory for the executor.
    pub fn with_memory(mut self, memory: Arc<RwLock<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Sets the options for the executor.
    pub fn with_options(mut self, options: ExecutorOptions) -> Self {
        self.options = options;
        self
    }

    /// Constructs a new [`ExecutionContext`] with the provided input and strategy.
    ///
    /// The [`ExecutionContext::start`] method can be called to actually begin the execution.
    ///
    /// ```ignore
    /// use langchain_rust::{agent::{ConversationalAgent, AgentExecutor, DefaultStrategy}, llm::{OpenAI, OpenAIModel}};
    /// use async_openai::config::OpenAIConfig;
    ///
    /// let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4o).build();
    ///
    /// let agent: ConversationalAgent = ConversationalAgent::builder()
    ///     .system_prompt("You are a helpful assistant.")
    ///     .initial_prompt("Help me find {{input}}.")
    ///     .build(llm);
    ///
    /// let executor = AgentExecutor::from_agent(agent);
    /// executor.execution("Input".into(), DefaultStrategy).start();
    /// ```
    pub fn execution<'exec, 'input, S: Strategy>(
        &'exec self,
        input: I::Target<'input>,
        strategy: S,
    ) -> ExecutionContext<'exec, 'agent, 'input, I, O, S> {
        ExecutionContext::new(self, input, strategy)
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Chain<I, O> for AgentExecutor<'_, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let output = self.execution(input, DefaultStrategy).start().await?;
        Ok(output.without_extra())
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<AgentInput<I::Target<'_>>> for AgentExecutor<'_, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError> {
        self.agent.get_prompt(&input)
    }
}
