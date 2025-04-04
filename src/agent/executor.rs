use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use indoc::indoc;
use tokio::sync::Mutex;

use super::{agent::Agent, AgentError, FinalAnswerValidator};
use crate::{
    chain::{chain_trait::Chain, ChainError},
    memory::Memory,
    schemas::{
        agent_plan::AgentEvent, AgentResult, GenerateResult, GenerateResultContent, InputVariables,
        Message, MessageType, TokenUsage, ToolCall,
    },
};

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct AgentExecutor {
    agent: Box<dyn Agent>,
    max_iterations: Option<usize>,
    max_consecutive_fails: Option<usize>,
    break_if_tool_error: bool,
    pub memory: Option<Arc<Mutex<dyn Memory>>>,
    final_answer_validator: Option<Box<dyn FinalAnswerValidator>>,
}

impl AgentExecutor {
    pub fn from_agent<A>(agent: A) -> Self
    where
        A: Agent + Send + Sync + 'static,
    {
        Self {
            agent: Box::new(agent),
            max_iterations: Some(10),
            max_consecutive_fails: Some(3),
            break_if_tool_error: false,
            memory: None,
            final_answer_validator: None,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    pub fn with_memory(mut self, memory: Arc<Mutex<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn with_break_if_tool_error(mut self, break_if_tool_error: bool) -> Self {
        self.break_if_tool_error = break_if_tool_error;
        self
    }

    pub fn with_final_answer_validator<V>(mut self, final_answer_validator: V) -> Self
    where
        V: FinalAnswerValidator + Send + Sync + 'static,
    {
        self.final_answer_validator = Some(Box::new(final_answer_validator));
        self
    }
}

#[async_trait]
impl Chain for AgentExecutor {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let mut steps: Vec<(ToolCall, String)> = Vec::new();
        let mut use_counts: HashMap<String, usize> = HashMap::new();
        let mut consecutive_fails: usize = 0;
        let mut total_usage: Option<TokenUsage> = None;

        if let Some(memory) = &self.memory {
            let memory: tokio::sync::MutexGuard<'_, dyn Memory> = memory.lock().await;
            input_variables.insert_placeholder_replacement("chat_history", memory.messages());
        // TODO: Possibly implement messages parsing
        } else {
            input_variables.insert_placeholder_replacement("chat_history", vec![]);
        }

        {
            let mut input_variables_demo = input_variables.clone();
            input_variables_demo.insert_placeholder_replacement("agent_scratchpad", vec![]);
            self.log_messages(&input_variables_demo).map_err(|e| {
                ChainError::AgentError(format!("Error formatting initial messages: {e}"))
            })?;
        }

        'step: loop {
            if self
                .max_consecutive_fails
                .is_some_and(|max_consecutive_fails| consecutive_fails >= max_consecutive_fails)
            {
                log::error!(
                    "Too many consecutive fails ({} in a row), aborting",
                    consecutive_fails
                );
                return Err(ChainError::AgentError("Too many consecutive fails".into()));
            }

            let AgentResult { content, usage } =
                match self.agent.plan(&steps, input_variables).await {
                    Ok(agent_event) => agent_event,
                    Err(e) => {
                        consecutive_fails += 1;
                        log::warn!("Error: {} ({} consecutive fails)", e, consecutive_fails);
                        continue 'step;
                    }
                };

            total_usage = match (total_usage, usage) {
                (None, None) => None,
                (Some(total_usage), None) => Some(total_usage),
                (None, Some(usage)) => Some(usage),
                (Some(total_usage), Some(usage)) => Some(total_usage.merge(&usage)),
            };

            match content {
                AgentEvent::Action(tool_calls) => {
                    if self
                        .max_iterations
                        .is_some_and(|max_iterations| steps.len() >= max_iterations)
                    {
                        log::warn!(
                            "Max iteration ({}) reached, forcing final answer",
                            self.max_iterations.unwrap()
                        );
                        input_variables.insert_placeholder_replacement(
                            "ultimatum",
                            vec![
                                Message::new(MessageType::AIMessage, ""),
                                Message::new(MessageType::HumanMessage, FORCE_FINAL_ANSWER),
                            ],
                        );
                        // TODO: Add ultimatum template
                        continue 'step;
                    }

                    for tool_call in tool_calls {
                        log::debug!("{}", tool_call);

                        let tool_name = tool_call.name.to_lowercase().replace(" ", "_");
                        let Some(tool) = self.agent.get_tool(&tool_name) else {
                            consecutive_fails += 1;
                            log::warn!(
                                "Agent tried to use nonexistent tool {}, retrying ({} consecutive fails)",
                                tool_call.name,
                                consecutive_fails
                            );
                            continue 'step;
                        };

                        if let Some(usage_limit) = tool.usage_limit() {
                            let count = use_counts.entry(tool_name.clone()).or_insert(0);
                            *count += 1;
                            if *count > usage_limit {
                                consecutive_fails += 1;
                                log::warn!(
                                    "Agent repeatedly using tool {} (usage limit: {}), preventing further use ({} consecutive fails)",
                                    tool_call.name,
                                    usage_limit,
                                    consecutive_fails
                                );
                                continue 'step;
                            }
                        }

                        let result = match tool.call(tool_call.arguments.clone()).await {
                            Ok(result) => result,
                            Err(e) => {
                                log::warn!(
                                    "Tool '{}' encountered an error: {}",
                                    &tool_call.name,
                                    e
                                );
                                if self.break_if_tool_error {
                                    return Err(ChainError::AgentError(
                                        AgentError::ToolError(e.to_string()).to_string(),
                                    ));
                                } else {
                                    format!(
                                        indoc! {"
                                            Tool call failed: {}
                                            If the error doesn't make sense to you, it means that the tool is broken. DO NOT use this tool again.
                                        "},
                                        e
                                    )
                                }
                            }
                        };

                        log::debug!("Tool {} result:\n{}", &tool_call.name, &result);

                        steps.push((tool_call, result));
                        consecutive_fails = 0;
                    }
                }
                AgentEvent::Finish(final_answer) => {
                    if let Some(validator) = &self.final_answer_validator {
                        if !validator.validate_final_answer(&final_answer, &steps) {
                            log::warn!(
                                "Final answer validation failed ({} consecutive fails)\nAnswer:{}",
                                consecutive_fails,
                                final_answer
                            );
                            continue 'step;
                        }
                    }

                    if let Some(memory) = &self.memory {
                        let mut memory = memory.lock().await;

                        memory.add_message(Message::new(
                            MessageType::HumanMessage,
                            input_variables
                                .get_text_replacement("input")
                                .unwrap_or(&String::new()),
                        ));

                        for (tool_call, observation) in steps {
                            memory.add_message(
                                Message::new(MessageType::AIMessage, "").with_tool_calls(vec![
                                    ToolCall {
                                        id: tool_call.id.clone(),
                                        name: tool_call.name,
                                        arguments: tool_call.arguments,
                                    },
                                ]),
                            );
                            memory.add_message(Message::new_tool_message::<_, &str>(
                                Some(&tool_call.id),
                                observation,
                            ));
                        }

                        memory.add_message(Message::new(
                            MessageType::AIMessage,
                            final_answer.clone(),
                        ));
                    }

                    log::debug!("Agent finished with result:\n{}", &final_answer);

                    return Ok(GenerateResult {
                        content: GenerateResultContent::Text(final_answer),
                        usage: total_usage,
                    });
                }
            }
        }
    }

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>> {
        self.agent.log_messages(inputs)
    }
}
