use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, error::Error};

use crate::schemas::generate_result::{GenerateResultContent, ToolCall};
use crate::{
    agent::{Agent, AgentError},
    chain::Chain,
    language_models::LLMError,
    prompt_template,
    schemas::{
        agent::{AgentAction, AgentEvent},
        InputVariables, Message, MessageType,
    },
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
    pub(crate) tools: HashMap<String, Arc<dyn Tool>>,
}

impl OpenAiToolAgent {
    pub fn create_prompt(
        system_prompt: &str,
        initial_prompt: &str,
    ) -> Result<PromptTemplate, AgentError> {
        let prompt = prompt_template![
            Message::new(MessageType::SystemMessage, system_prompt),
            MessageOrTemplate::Placeholder("chat_history".into()),
            MessageTemplate::from_jinja2(MessageType::HumanMessage, initial_prompt),
            MessageOrTemplate::Placeholder("agent_scratchpad".into())
        ];

        Ok(prompt)
    }

    fn construct_scratchpad(&self, intermediate_steps: &[(AgentAction, String)]) -> Vec<Message> {
        intermediate_steps
            .iter()
            .flat_map(|(action, observation)| {
                vec![
                    Message::new(MessageType::AIMessage, "").with_tool_calls(vec![ToolCall {
                        id: action.id.clone(),
                        name: action.action.clone(),
                        arguments: action.action_input.clone(),
                    }]),
                    Message::new_tool_message(Some(action.id.clone()), observation),
                ]
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl Agent for OpenAiToolAgent {
    async fn plan(
        &self,
        intermediate_steps: &[(AgentAction, String)],
        inputs: &mut InputVariables,
    ) -> Result<AgentEvent, AgentError> {
        let scratchpad = self.construct_scratchpad(intermediate_steps);
        inputs.insert_placeholder_replacement("agent_scratchpad", scratchpad);
        let output = self.chain.call(inputs).await?;

        match output.content {
            GenerateResultContent::Text(text) => Ok(AgentEvent::Finish(text)),
            GenerateResultContent::ToolCall(tool_calls) => {
                let agent_actions = tool_calls
                    .into_iter()
                    .map(|tool_call: ToolCall| AgentAction {
                        id: tool_call.id,
                        action: tool_call.name,
                        action_input: tool_call.arguments,
                    })
                    .collect::<Vec<_>>();
                Ok(AgentEvent::Action(agent_actions))
            }
            GenerateResultContent::Refusal(refusal) => {
                return Err(AgentError::LLMError(LLMError::OtherError(format!(
                    "LLM refused to answer: {}",
                    refusal
                ))));
            }
        }
    }

    fn get_tool(&self, tool_name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(tool_name).cloned()
    }

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>> {
        self.chain.log_messages(inputs)
    }
}
