use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::ExecutorOptions;
use crate::agent::{Agent, DefaultStrategy, ExecutionContext, Strategy};
use crate::chain::{Chain, ChainError, ChainOutput, GetPrompt, InputCtor, OutputCtor};
use crate::memory::Memory;
use crate::schemas::{Prompt, WithUsage};
use crate::template::TemplateError;

/// A runtime executor for driving multi-step agent execution with memory, planning, and tool use.
///
/// This struct coordinates the full reasoning loop of an [`Agent`](crate::agent::Agent), handling
/// prompt generation, scratchpad construction, tool resolution, and optional memory integration. It
/// provides a high-level interface for running agents in a predictable, type-safe, and optionally
/// stateful manner.
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type.
pub struct AgentExecutor<'tool, I: InputCtor, O: OutputCtor>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    /// The agent used by the executor.
    pub(super) agent: Agent<'tool, I, O>,
    /// The memory that stores messages across executions.
    pub(super) memory: Option<Arc<RwLock<dyn Memory>>>,
    /// The execution options.
    pub(super) options: ExecutorOptions,
}

impl<'tool, I: InputCtor, O: OutputCtor> AgentExecutor<'tool, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    /// Constructs a new [`AgentExecutor`] from a struct that implements the trait [`Agent`].
    pub fn from_agent(agent: Agent<'tool, I, O>) -> Self {
        Self {
            agent,
            memory: None,
            options: ExecutorOptions::default(),
        }
    }

    /// Returns the unique identifier of the agent.
    pub fn agent_id(&self) -> &str {
        &self.agent.id
    }

    /// Returns a reference to the memory, if any.
    pub fn get_memory(&self) -> Option<Arc<RwLock<dyn Memory>>> {
        self.memory.clone()
    }

    /// Attaches a memory to the executor.
    pub fn with_memory(mut self, memory: Arc<RwLock<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Sets the execution options.
    pub fn with_options(mut self, options: ExecutorOptions) -> Self {
        self.options = options;
        self
    }

    /// Constructs a new [`ExecutionContext`] with the provided input.
    ///
    /// The default strategy is used for the execution.
    ///
    /// The [`ExecutionContext::start`] method can then be called to begin the execution.
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
    /// executor.execution("Input".into()).start();
    /// ```
    pub fn execution<'exec, 'input>(
        &'exec self,
        input: I::Target<'input>,
    ) -> ExecutionContext<'exec, 'tool, 'input, I, O, DefaultStrategy> {
        ExecutionContext::new(self, input, DefaultStrategy)
    }

    /// Constructs a new [`ExecutionContext`] with the provided input and custom strategy.
    ///
    /// The [`ExecutionContext::start`] method can then be called to begin the execution.
    pub fn execution_with_strategy<'exec, 'input, S: Strategy>(
        &'exec self,
        input: I::Target<'input>,
        strategy: S,
    ) -> ExecutionContext<'exec, 'tool, 'input, I, O, S> {
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
        let output = self.execution(input).start().await?;
        Ok(output.without_extra())
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<I::Target<'_>> for AgentExecutor<'_, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.agent.get_prompt(input)
    }
}
