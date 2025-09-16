use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use crate::agent::{
    AgentBuilder, AgentError, AgentExecutor, AgentInput, AgentInputCtor, AgentOutputCtor, AgentStep,
};
use crate::chain::{
    ChainOutput, DefaultChainInputCtor, GetPrompt, InputCtor, LLMChain, OutputCtor, StringCtor,
};
use crate::schemas::{Message, Prompt};
use crate::template::TemplateError;
use crate::tools::{Tool, Toolbox};

/// An agent implementation for agents that do **not** support structured tool calling.
///
/// This agent enables tool use by prompting the model to emit tool call as plain text, which are
/// then manually parsed into tool invocations. You can also provide a custom
/// [`Instructor`](crate::instructor::Instructor) to customize the tool call format instruction and
/// parsing logic.
///
/// While this works with any language models, it is more error-prone compared to structured tool
/// call. For OpenAI models that support structured tool calls, consider using
/// [`OpenAiToolAgent`](crate::agent::OpenAiToolAgent).
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type (defaults to
///   [`DefaultChainInputCtor`], which constructs [`ChainInput`](crate::chain::DefaultChainInput)).
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type (defaults to
///   [`StringCtor`], which constructs [`String`]).
pub struct Agent<'tool, I: InputCtor = DefaultChainInputCtor, O: OutputCtor = StringCtor> {
    pub(super) id: String,
    /// The inner [`LLMChain`] used for prompt construction and LLM invocation.
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
    /// A map of registered tool names to their implementations.
    pub(super) tools: HashMap<String, Tool<'tool>>,
    /// A list of toolboxes used to dynamically provide tools at runtime.
    pub(super) toolboxes: Vec<Arc<dyn Toolbox>>, /* Has to be Arc because ownership needs to be
                                                  * shared with ListTools */
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<'tool, I: InputCtor, O: OutputCtor> Agent<'tool, I, O> {
    /// Creates a new `AgentStruct` with the given LLM chain and tools.
    ///
    /// # Arguments
    /// - `llm_chain`: The [`LLMChain`] to use for prompt construction and LLM invocation.
    /// - `tools`: A vector of [`Tool`]s that the agent can use.
    ///
    /// # Returns
    /// A new instance of `AgentStruct`.
    pub fn new(
        id: String,
        llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
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

    pub fn executor(self) -> AgentExecutor<'tool, I, O>
    where
        for<'any> I::Target<'any>: Display,
        for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
    {
        AgentExecutor::from_agent(self)
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Converts prior reasoning steps into a sequence of messages used to populate the prompt.
    ///
    /// Invoked by the [`AgentExecutor`] at each step before `plan` is called, this method takes a
    /// sequence of [`AgentStep`]s—each representing a completed tool call and its result—and
    /// transforms them into a sequence of [`Message`]s (typically alternating between assistant
    /// and tool messages).
    ///
    /// # Arguments
    /// - `steps`: A list of previously completed reasoning steps, each containing a tool call and
    ///   its result.
    ///
    /// # Returns
    /// A vector of [`Message`]s suitable for inclusion in the LLM prompt, or an [`AgentError`] if
    /// rendering fails.
    pub fn construct_scratchpad(&self, steps: &[AgentStep]) -> Vec<Message> {
        steps
            .iter()
            .flat_map(|step| {
                [
                    Message::new_tool_call_message([step.tool_call.clone()]),
                    Message::new_tool_message(Some(&step.tool_call.id), &step.result),
                ]
            })
            .collect::<Vec<_>>()
    }

    pub fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }

    pub fn log_initial_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<(), AgentError> {
        if !log::log_enabled!(log::Level::Debug) {
            return Ok(());
        }

        for message in self.get_prompt(input)?.to_messages() {
            log::debug!(
                "{}:\n{}",
                message.message_type.to_string().to_uppercase(),
                message.content
            );
        }
        Ok(())
    }
}
