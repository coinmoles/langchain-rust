use async_trait::async_trait;
use std::collections::HashMap;

use crate::{
    agent::{
        Agent, AgentError, AgentInput, AgentInputCtor, AgentOutput, AgentOutputCtor, AgentStep,
    },
    chain::{DefaultChainInputCtor, InputCtor, LLMChain, OutputCtor, StringCtor},
    schemas::{GetPrompt, Message, Prompt, WithUsage},
    template::TemplateError,
    tools::{ToolDyn, Toolbox},
};

use super::OpenAiToolAgentBuilder;

/// An agent implementation for OpenAI models that support OpenAI's structured tool calling.
///
/// While all models can be used with [`crate::agent::ConversationalAgent`], this implementation
/// is recommended as it takes advantage of OpenAI's structured tool calling capabilities,
/// without having to rely on manual parsing.
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type (defaults to
///   [`DefaultChainInputCtor`], which constructs [`ChainInput`](crate::chain::DefaultChainInput)).
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type (defaults to
///   [`StringCtor`], which constructs [`String`]).
pub struct OpenAiToolAgent<I: InputCtor = DefaultChainInputCtor, O: OutputCtor = StringCtor> {
    /// The inner [`LLMChain`] used for prompt construction and LLM invocation.
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
    /// A map of registered tool names to their implementations.
    pub(super) tools: HashMap<String, Box<dyn ToolDyn>>,
    /// A list of toolboxes used to dynamically provide tools at runtime.
    pub(super) toolboxes: Vec<Box<dyn Toolbox>>,
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<I: InputCtor, O: OutputCtor> OpenAiToolAgent<I, O> {
    /// Creates a [`OpenAiToolAgentBuilder`] to configure an [`OpenAiToolAgent`].
    ///
    /// This is the same as calling [`OpenAiToolAgentBuilder::new()`].
    ///
    /// # Example:
    /// ```
    /// use langchain_rust::{agent::OpenAiToolAgent, llm::{OpenAI, OpenAIModel}};
    /// use async_openai::config::OpenAIConfig;
    ///
    /// let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4o).build();
    ///
    /// let agent: OpenAiToolAgent = OpenAiToolAgent::builder()
    ///     .system_prompt("You are a helpful assistant.")
    ///     .initial_prompt("Help me find {{input}}.")
    ///     // .tools(vec![my_tool]) // You can add tools here
    ///     .build(llm);
    /// ```
    #[must_use]
    pub fn builder<'a, 'b>() -> OpenAiToolAgentBuilder<'a, 'b, I, O> {
        OpenAiToolAgentBuilder::new()
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Agent<I, O> for OpenAiToolAgent<I, O> {
    async fn construct_scratchpad(&self, steps: &[AgentStep]) -> Result<Vec<Message>, AgentError> {
        let scratchpad = steps
            .iter()
            .flat_map(|step| {
                [
                    Message::new_tool_call_message([step.tool_call.clone()]),
                    Message::new_tool_message(Some(&step.tool_call.id), &step.result),
                ]
            })
            .collect::<Vec<_>>();
        Ok(scratchpad)
    }

    async fn plan<'i>(
        &self,
        input: &AgentInput<I::Target<'i>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError> {
        let plan = self.llm_chain.call_with_reference(input).await?;
        Ok(plan)
    }

    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolDyn> {
        if let Some(tool) = self.tools.get(tool_name).map(|t| t.as_ref()) {
            return Some(tool);
        }

        for toolbox in &self.toolboxes {
            if let Some(tool) = toolbox.get_tool(tool_name) {
                return Some(tool);
            }
        }

        None
    }

    fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<I::Target<'_>> for OpenAiToolAgent<I, O> {
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(&AgentInput::new(input))
    }
}
