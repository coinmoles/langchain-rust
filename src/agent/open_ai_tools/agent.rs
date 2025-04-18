use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error};

use crate::schemas::{AgentResult, GenerateResultContent, ToolCall};
use crate::tools::Toolbox;
use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    language_models::LLMError,
    prompt_template,
    schemas::{agent_plan::AgentEvent, InputVariables, Message, MessageType},
    template::{MessageOrTemplate, MessageTemplate, PromptTemplate},
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
    pub fn create_prompt(
        system_prompt: &str,
        initial_prompt: &str,
    ) -> Result<PromptTemplate, AgentError> {
        let prompt = prompt_template![
            MessageTemplate::from_jinja2(MessageType::SystemMessage, system_prompt),
            MessageOrTemplate::Placeholder("chat_history".into()),
            MessageTemplate::from_jinja2(MessageType::HumanMessage, initial_prompt),
            MessageOrTemplate::Placeholder("agent_scratchpad".into()),
            MessageOrTemplate::Placeholder("ultimatum".into())
        ];

        Ok(prompt)
    }

    fn construct_scratchpad(&self, intermediate_steps: &[(ToolCall, String)]) -> Vec<Message> {
        intermediate_steps
            .iter()
            .flat_map(|(tool_call, result)| {
                vec![
                    Message::new_tool_call_message([tool_call.clone()]),
                    Message::new_tool_message(Some(&tool_call.id), result.clone()),
                ]
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl Agent for OpenAiToolAgent {
    async fn plan(
        &self,
        intermediate_steps: &[(ToolCall, String)],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError> {
        let scratchpad = self.construct_scratchpad(intermediate_steps);
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

    async fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool> {
        if let Some(tool) = self.tools.get(tool_name).map(|t| t.as_ref()) {
            return Some(tool);
        }

        for toolbox in &self.toolboxes {
            if let Ok(tool) = toolbox.get_tool(tool_name).await {
                return Some(tool);
            }
        }

        None
    }

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>> {
        self.chain.log_messages(inputs)
    }
}
