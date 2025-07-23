use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

use crate::{
    agent::{
        Agent, AgentError, AgentInput, AgentInputCtor, AgentOutput, AgentOutputCtor, AgentStep,
    },
    chain::{ChainOutput, DefaultChainInputCtor, InputCtor, LLMChain, OutputCtor, StringCtor},
    schemas::{GetPrompt, Message, Prompt, WithUsage},
    template::TemplateError,
    tools::{ToolDyn, Toolbox},
};

use super::OpenAiToolAgentBuilder;

///Log tools is a struct used by the openai-like agents
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogTools {
    pub tool_id: String,
    pub tools: String,
}

pub struct OpenAiToolAgent<I: InputCtor = DefaultChainInputCtor, O: OutputCtor = StringCtor>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
    pub(super) tools: HashMap<String, Box<dyn ToolDyn>>,
    pub(super) toolboxes: Vec<Box<dyn Toolbox>>,
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<I: InputCtor, O: OutputCtor> OpenAiToolAgent<I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn builder<'a, 'b>() -> OpenAiToolAgentBuilder<'a, 'b, I, O> {
        OpenAiToolAgentBuilder::new()
    }

    fn construct_scratchpad(&self, steps: &[AgentStep]) -> Vec<Message> {
        steps
            .iter()
            .flat_map(|step| {
                vec![
                    Message::new_tool_call_message([step.tool_call.clone()]),
                    Message::new_tool_message(Some(&step.tool_call.id), step.result.clone()),
                ]
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Agent<I, O> for OpenAiToolAgent<I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<I::Target<'i>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError> {
        input.set_agent_scratchpad(self.construct_scratchpad(steps));
        let result = self.llm_chain.call_with_reference(input).await?;
        Ok(result)
    }

    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolDyn> {
        if let Some(tool) = self.tools.get(tool_name).map(|t| t.as_ref()) {
            return Some(tool);
        }

        for toolbox in &self.toolboxes {
            if let Ok(tool) = toolbox.get_tool(tool_name) {
                return Some(tool);
            }
        }

        None
    }

    fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<I::Target<'_>> for OpenAiToolAgent<I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(&AgentInput::new(input))
    }
}
