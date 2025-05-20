use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use crate::schemas::{AgentResult, AgentStep, GenerateResultContent, Prompt};
use crate::tools::Toolbox;
use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    language_models::LLMError,
    schemas::{agent_plan::AgentEvent, InputVariables, Message},
    tools::Tool,
};

///Log tools is a struct used by the openai-like agents
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogTools {
    pub tool_id: String,
    pub tools: String,
}

pub struct OpenAiToolAgent {
    pub(crate) chain: Box<dyn Chain>,
    pub(crate) tools: HashMap<String, Box<dyn Tool>>,
    pub(crate) toolboxes: Vec<Box<dyn Toolbox>>,
}

impl OpenAiToolAgent {
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
        let output = self.chain.call(inputs).await?;

        let content = match output.content {
            GenerateResultContent::Text(text) => AgentEvent::Finish(text),
            GenerateResultContent::ToolCall(tool_calls) => AgentEvent::Action(tool_calls),
            GenerateResultContent::Refusal(refusal) => {
                return Err(AgentError::LLMError(LLMError::OtherError(format!(
                    "LLM refused to answer: {}",
                    refusal
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
        self.chain.get_prompt(inputs)
    }
}
