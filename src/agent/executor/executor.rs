use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use indoc::indoc;
use tokio::sync::RwLock;

use crate::agent::AgentError;
use crate::schemas::Prompt;
use crate::{
    agent::Agent,
    chain::{chain_trait::Chain, ChainError},
    memory::Memory,
    schemas::{
        agent_plan::AgentEvent, AgentResult, AgentStep, GenerateResult, GenerateResultContent,
        InputVariables, Message, TokenUsage,
    },
};

use super::ExecutorOptions;

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct AgentExecutor<'a> {
    agent: Box<dyn Agent + 'a>,
    memory: Option<Arc<RwLock<dyn Memory>>>,
    options: ExecutorOptions,
}

impl<'a> AgentExecutor<'a> {
    pub fn from_agent(agent: impl Agent + 'a) -> Self {
        Self {
            agent: Box::new(agent),
            memory: None,
            options: ExecutorOptions::default(),
        }
    }

    pub fn with_memory(mut self, memory: Arc<RwLock<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn with_options(mut self, options: ExecutorOptions) -> Self {
        self.options = options;
        self
    }
}

#[async_trait]
impl Chain for AgentExecutor<'_> {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let options = &self.options;

        let mut steps: Vec<AgentStep> = Vec::new();
        let mut use_counts: HashMap<String, usize> = HashMap::new();
        let mut consecutive_fails: usize = 0;
        let mut total_usage: Option<TokenUsage> = None;

        if let Some(memory) = &self.memory {
            let memory = memory.read().await;
            input_variables.insert_placeholder_replacement("chat_history", memory.messages());
        // TODO: Possibly implement messages parsing
        } else {
            input_variables.insert_placeholder_replacement("chat_history", vec![]);
        }

        {
            let mut input_variables_demo = input_variables.clone();
            input_variables_demo.insert_placeholder_replacement("agent_scratchpad", vec![]);
            let prompt = self.get_prompt(&input_variables_demo).map_err(|e| {
                ChainError::AgentError(format!("Error formatting initial messages: {e}"))
            })?;
            for message in prompt.to_messages() {
                log::debug!(
                    "\n{}:\n{}",
                    message.message_type.to_string().to_uppercase(),
                    message.content
                );
            }
        }

        'step: loop {
            if options
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

            total_usage = TokenUsage::merge_options(total_usage, usage);

            match content {
                AgentEvent::Action(tool_calls) => {
                    if let Some(max_iterations) = options.max_iterations {
                        if steps.len() >= max_iterations {
                            log::warn!(
                                "Max iteration ({}) reached, forcing final answer",
                                max_iterations
                            );
                            input_variables.insert_placeholder_replacement(
                                "ultimatum",
                                vec![
                                    Message::new_ai_message(""),
                                    Message::new_human_message(FORCE_FINAL_ANSWER),
                                ],
                            );
                            continue 'step;
                        }
                    }

                    for tool_call in tool_calls {
                        log::debug!("\nTool call:\n{}", tool_call);

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
                                if options.break_if_tool_error {
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

                        log::debug!("\nTool {} result:\n{}", &tool_call.name, &result);

                        let agent_step = AgentStep::new(tool_call, result);

                        steps.push(agent_step);
                        consecutive_fails = 0;
                    }
                }
                AgentEvent::Finish(final_answer) => {
                    if let Some(validator) = &options.final_answer_validator {
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
                        let mut memory = memory.write().await;

                        memory.add_human_message(
                            input_variables
                                .get_text_replacement("input")
                                .cloned()
                                .unwrap_or_default(),
                        );

                        for step in steps {
                            memory.add_tool_call_message(vec![step.tool_call.clone()]);
                            memory.add_tool_message(
                                Some(step.tool_call.id.clone()),
                                step.result.clone(),
                            );
                        }

                        memory.add_ai_message(final_answer.clone());
                    }

                    log::debug!("\nAgent finished with result:\n{}", &final_answer);

                    return Ok(GenerateResult {
                        content: GenerateResultContent::Text(final_answer),
                        usage: total_usage,
                    });
                }
            }
        }
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error>> {
        self.agent.get_prompt(inputs)
    }
}
