use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

use crate::agent::{AgentInput, AgentInputCtor};
use crate::chain::{ChainError, LLMChain};
use crate::schemas::{
    AgentStep, ChainOutput, DefaultChainInputCtor, InputCtor, IntoWithUsage, LLMOutput, OutputCtor,
    Prompt, StringCtor, WithUsage,
};
use crate::tools::Toolbox;
use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    language_models::LLMError,
    schemas::{agent_plan::AgentEvent, Message},
    tools::Tool,
};

use super::OpenAiToolAgentBuilder;

///Log tools is a struct used by the openai-like agents
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogTools {
    pub tool_id: String,
    pub tools: String,
}

pub struct OpenAiToolAgent<I = DefaultChainInputCtor, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
    for<'c> I::Target<'c>: Display,
    for<'c> O::Target<'c>: ChainOutput<AgentInput<I::Target<'c>>>,
{
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, O>,
    pub(super) tools: HashMap<String, Box<dyn Tool>>,
    pub(super) toolboxes: Vec<Box<dyn Toolbox>>,
}

impl<I, O> OpenAiToolAgent<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'c> I::Target<'c>: Display,
    for<'c> O::Target<'c>: ChainOutput<AgentInput<I::Target<'c>>>,
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
impl<I, O> Agent for OpenAiToolAgent<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'c> I::Target<'c>: Display,
    for<'c> O::Target<'c>: ChainOutput<AgentInput<I::Target<'c>>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<I::Target<'i>>,
    ) -> Result<WithUsage<AgentEvent>, AgentError> {
        let scratchpad = self.construct_scratchpad(steps);
        input.set_agent_scratchpad(scratchpad);
        let output = self.llm_chain.call_llm(input).await?;

        let content = match output.content {
            LLMOutput::Text(text) => AgentEvent::Finish(text),
            LLMOutput::ToolCall(tool_calls) => AgentEvent::Action(tool_calls),
            LLMOutput::Refusal(refusal) => {
                return Err(AgentError::LLMError(LLMError::OtherError(format!(
                    "LLM refused to answer: {refusal}"
                ))));
            }
        };
        let usage = output.usage;

        Ok(content.with_usage(usage))
    }

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool> {
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

    fn get_prompt<'i>(
        &self,
        input: AgentInput<<Self::InputCtor as InputCtor>::Target<'i>>,
    ) -> Result<Prompt, ChainError> {
        self.llm_chain.get_prompt(input)
    }
}
