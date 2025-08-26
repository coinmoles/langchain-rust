use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;

use crate::{
    agent::{
        Agent, AgentError, AgentInput, AgentInputCtor, AgentOutput, AgentOutputCtor, AgentStep,
    },
    chain::{DefaultChainInputCtor, InputCtor, LLMChain, OutputCtor, StringCtor},
    schemas::{GetPrompt, Message, Prompt, WithUsage},
    template::TemplateError,
    tools::{ToolDyn, Toolbox},
};

use super::ConversationalAgentBuilder;

/// An agent implementation for agents that do **not** support structured tool calling.
///
/// This agent enables tool use by prompting the model to emit tool call as plain text, which are then
/// manually parsed into tool invocations. You can also provide a custom
/// [`Instructor`](crate::instructor::Instructor) to customize the tool call format instruction and parsing logic.
///
/// While this works with any language models, it is more error-prone compared to structured tool call.
/// For OpenAI models that support structured tool calls, consider using
/// [`OpenAiToolAgent`](crate::agent::OpenAiToolAgent).
///
/// # Type Parameters
/// - `I`: A [constructor](crate::chain::Ctor) for the agent’s input type (defaults to
///   [`DefaultChainInputCtor`], which constructs [`ChainInput`](crate::chain::DefaultChainInput)).
/// - `O`: A [constructor](crate::chain::Ctor) for the agent’s output type (defaults to
///   [`StringCtor`], which constructs [`String`]).
pub struct ConversationalAgent<I: InputCtor = DefaultChainInputCtor, O: OutputCtor = StringCtor> {
    /// The inner [`LLMChain`] used for prompt construction and LLM invocation.
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
    /// A map of registered tool names to their implementations.
    pub(super) tools: HashMap<String, Box<dyn ToolDyn>>,
    /// A list of toolboxes used to dynamically provide tools at runtime.
    pub(super) toolboxes: Vec<Arc<dyn Toolbox>>, // Has to be Arc because ownership needs to be shared with ListTools
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<I: InputCtor, O: OutputCtor> ConversationalAgent<I, O> {
    /// Creates a [`ConversationalAgentBuilder`] to configure an [`ConversationalAgent`].
    ///
    /// This is the same as calling [`ConversationalAgentBuilder::new()`].
    ///
    /// # Example:
    /// ```
    /// use langchain_rust::{agent::ConversationalAgent, llm::{OpenAI, OpenAIModel}};
    /// use async_openai::config::OpenAIConfig;
    ///
    /// let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4o).build();
    ///
    /// let agent: ConversationalAgent = ConversationalAgent::builder()
    ///     .system_prompt("You are a helpful assistant.")
    ///     .initial_prompt("Help me find {{input}}.")
    ///     // .tools(vec![my_tool]) // You can add tools here
    ///     .build(llm);
    /// ```
    pub fn builder<'a, 'b>() -> ConversationalAgentBuilder<'a, 'b, I, O> {
        ConversationalAgentBuilder::new()
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Agent<I, O> for ConversationalAgent<I, O> {
    async fn construct_scratchpad(&self, steps: &[AgentStep]) -> Result<Vec<Message>, AgentError> {
        let scratchpad = steps
            .iter()
            .flat_map(|step| {
                [
                    Message::new_ai_message(&step.tool_call),
                    Message::new_human_message(&step.result),
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

#[cfg(test)]
mod tests {
    use std::error::Error;

    use async_openai::config::OpenAIConfig;
    use async_trait::async_trait;
    use serde_json::Value;

    use crate::{
        agent::{Agent, ConversationalAgent},
        chain::{Chain, DefaultChainInput},
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        tools::Tool,
    };

    #[derive(Default)]
    struct Calc {}

    #[async_trait]
    impl Tool for Calc {
        type Input = String;
        type Output = String;

        fn name(&self) -> String {
            "Calculator".to_string()
        }
        fn description(&self) -> String {
            "Useful for making calculations".to_string()
        }
        async fn parse_input(&self, input: Value) -> Result<String, serde_json::Error> {
            Ok(input.to_string())
        }
        async fn run(
            &self,
            _input: Self::Input,
        ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
            Ok("25".to_string())
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_invoke_agent() {
        let llm: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt4.to_string())
            .build();
        let memory = SimpleMemory::new();
        let tool_calc = Calc::default();
        let agent: ConversationalAgent =
            ConversationalAgent::builder().tools([tool_calc]).build(llm);
        let input = DefaultChainInput::new(
            "hola,Me llamo luis, y tengo 10 anos, y estudio Computer scinence",
        );
        let executor = agent.executor().with_memory(memory.into());
        match executor.call(input).await {
            Ok(result) => println!("Result: {:?}", result.content),
            Err(e) => panic!("Error invoking LLMChain: {e:?}"),
        }

        let input = DefaultChainInput::new("cuanta es la edad de luis +10 y que estudia");
        match executor.call(input).await {
            Ok(result) => println!("Result: {:?}", result.content),
            Err(e) => panic!("Error invoking LLMChain: {e:?}"),
        }
    }
}
