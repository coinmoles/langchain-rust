use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use crate::chain::LLMChain;
use crate::schemas::{AgentResult, AgentStep, LLMOutput, Prompt};
use crate::tools::Toolbox;
use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    language_models::LLMError,
    schemas::{agent_plan::AgentEvent, InputVariables, Message},
    tools::Tool,
};

use super::OpenAiToolAgentBuilder;

///Log tools is a struct used by the openai-like agents
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogTools {
    pub tool_id: String,
    pub tools: String,
}

pub struct OpenAiToolAgent {
    pub(super) llm_chain: LLMChain,
    pub(super) tools: HashMap<String, Box<dyn Tool>>,
    pub(super) toolboxes: Vec<Box<dyn Toolbox>>,
}

impl OpenAiToolAgent {
    pub fn builder<'a, 'b>() -> OpenAiToolAgentBuilder<'a, 'b> {
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
impl Agent for OpenAiToolAgent {
    async fn plan(
        &self,
        steps: &[AgentStep],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError> {
        let scratchpad = self.construct_scratchpad(steps);
        inputs.insert_placeholder_replacement("agent_scratchpad", scratchpad);
        let output = self.llm_chain.call_llm(inputs).await?;

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

        Ok(AgentResult { content, usage })
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

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>> {
        self.llm_chain.get_prompt(inputs)
    }
}
