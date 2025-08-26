use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    agent::{AgentOutput, AgentStep},
    chain::{ChainOutput, InputCtor, OutputCtor},
    schemas::{Message, Prompt, WithUsage},
    template::TemplateError,
    tools::ToolDyn,
};

use super::{AgentError, AgentExecutor, AgentInput};

/// Defines the interface for an agent capable of reasoning and tool usage within an [`AgentExecutor`] framework.
///
/// While this trait defines the core functionality for agents, agents are typically **not used on their own**.
/// Instead, they are wrapped and driven by an [`AgentExecutor`], which orchestrates the full execution loop and
/// manages state transitions.
///
/// The [`Agent::executor`] method offers a convenient way to wrap an agent into an executable [`AgentExecutor`].
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the input type.
/// - `O`: A [constructor](crate::chain::Ctor) for the output type.
#[async_trait]
pub trait Agent<I: InputCtor, O: OutputCtor>: Send + Sync {
    /// Converts prior reasoning steps into a sequence of messages used to populate the prompt.
    ///
    /// Invoked by the [`AgentExecutor`] at each step before `plan` is called, this method takes a sequence of
    /// [`AgentStep`]s—each representing a completed tool call and its result—and transforms them into a
    /// sequence of [`Message`]s (typically alternating between assistant and tool messages).
    ///
    /// # Arguments
    /// - `steps`: A list of previously completed reasoning steps, each containing a tool call and its result.
    ///
    /// # Returns
    /// A vector of [`Message`]s suitable for inclusion in the LLM prompt, or an [`AgentError`] if rendering fails.
    async fn construct_scratchpad(&self, steps: &[AgentStep]) -> Result<Vec<Message>, AgentError>;

    /// Determines the agent’s next action based on the current input and internal reasoning strategy.
    ///
    /// Invoked by the [`AgentExecutor`] at each step of the reasoning process, this method produces a plan
    /// for the next action—either a tool call or a final output.
    ///
    /// # Arguments
    /// - `input`: The current [`AgentInput`].
    ///
    /// # Returns
    /// A [`WithUsage<AgentOutput>`] containing the planned output and token usage,
    /// or an [`AgentError`] if planning fails.
    async fn plan<'a>(
        &self,
        input: &AgentInput<I::Target<'a>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError>;

    /// Resolves a tool by name for use during agent execution.
    ///
    /// Invoked by the [`AgentExecutor`] when the agent plans to call a tool,
    /// this method returns a reference to the corresponding tool implementation, if available.
    ///
    /// # Arguments
    /// - `tool_name`: The identifier of the tool to retrieve.
    ///
    /// # Returns
    /// An optional reference to a [`ToolDyn`] trait object, or [`None`] if the tool is not found.
    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolDyn>;

    /// Generates the prompt for the agent based on the current input.
    ///
    /// Invoked by the [`AgentExecutor`] before any planning step, this method constructs a [`Prompt`]
    /// from the given [`AgentInput`]. Which is then used for logging and debugging.
    ///
    /// # Arguments
    /// - `input`: The current [`AgentInput`].
    ///
    /// # Returns
    /// A rendered [`Prompt`] or a [`TemplateError`] if prompt construction fails.
    fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError>;

    /// A helper method that wraps the agent into an [`AgentExecutor`] for execution.
    ///
    /// # Returns
    /// An [`AgentExecutor`] configured to run the agent.
    fn executor<'a>(self) -> AgentExecutor<'a, I, O>
    where
        Self: Sized + 'a,
        for<'any> I::Target<'any>: Display,
        for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
    {
        AgentExecutor::from_agent(self)
    }
}
