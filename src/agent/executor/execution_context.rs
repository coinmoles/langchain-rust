use std::collections::HashMap;

use indoc::formatdoc;

use crate::{
    agent::AgentError,
    chain::ChainError,
    schemas::{
        AgentEvent, AgentResult, AgentStep, GenerateResult, GenerateResultContent, InputVariables,
        Message, OnStepFunc, TokenUsage,
    },
    tools::Tool,
    utils::helper::normalize_tool_name,
};

use super::AgentExecutor;

const FORCE_FINAL_ANSWER: &str = "Now it's time you MUST give your absolute best final answer. You'll ignore all previous instructions, stop using any tools, and just return your absolute BEST Final answer.";

pub struct ExecutionContext<'a, 'b> {
    executor: &'a AgentExecutor<'a>,
    on_step_func: Option<Box<OnStepFunc>>,
    shadow_tools: HashMap<String, Box<dyn Tool + 'b>>,
}

impl<'a, 'b> ExecutionContext<'a, 'b> {
    pub fn new(executor: &'a AgentExecutor) -> Self {
        Self {
            executor,
            on_step_func: None,
            shadow_tools: HashMap::new(),
        }
    }

    pub fn with_on_step_func(mut self, on_step_func: Box<OnStepFunc>) -> Self {
        self.on_step_func = Some(on_step_func);
        self
    }

    pub async fn with_shadow_tool(mut self, tool: Box<dyn Tool + 'b>) -> Self {
        let name = normalize_tool_name(&tool.name());

        if self.executor.agent.get_tool(&name).await.is_none() {
            log::warn!(
                "Tool {} doesn't exist in the agent, this tool will likely not work",
                name
            );
        }

        self.shadow_tools.insert(name, tool);
        self
    }

    async fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        if let Some(tool) = self.shadow_tools.get(name) {
            return Some(tool.as_ref());
        }

        self.executor.agent.get_tool(name).await
    }

    pub async fn start(
        mut self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let options = &self.executor.options;

        let mut steps: Vec<AgentStep> = Vec::new();
        let mut use_counts: HashMap<String, usize> = HashMap::new();
        let mut consecutive_fails: usize = 0;
        let mut total_usage: Option<TokenUsage> = None;

        if let Some(memory) = &self.executor.memory {
            let memory = memory.read().await;
            input_variables.insert_placeholder_replacement("chat_history", memory.messages());
        // TODO: Possibly implement messages parsing
        } else {
            input_variables.insert_placeholder_replacement("chat_history", vec![]);
        }

        {
            let mut input_variables_demo = input_variables.clone();
            input_variables_demo.insert_placeholder_replacement("agent_scratchpad", vec![]);
            self.executor
                .agent
                .log_messages(&input_variables_demo)
                .map_err(|e| {
                    ChainError::AgentError(format!("Error formatting initial messages: {e}"))
                })?;
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
                match self.executor.agent.plan(&steps, input_variables).await {
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
                        log::debug!("{}", tool_call);

                        let tool_name = tool_call.name.to_lowercase().replace(" ", "_");
                        let Some(tool) = self.get_tool(&tool_name).await else {
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
                                    formatdoc! {"
                                        Tool call failed: {}
                                        If the error doesn't make sense to you, it means that the tool is broken. DO NOT use this tool again.", 
                                        e
                                    }
                                }
                            }
                        };

                        log::debug!("Tool {} result:\n{}", &tool_call.name, &result);

                        let agent_step = AgentStep::new(tool_call, result);

                        if let Some(step_func) = &mut self.on_step_func {
                            if let Err(e) = step_func(&agent_step).await {
                                log::warn!("Error in step function: {}", e);
                            };
                        }

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

                    if let Some(memory) = &self.executor.memory {
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

                    log::debug!("Agent finished with result:\n{}", &final_answer);

                    return Ok(GenerateResult {
                        content: GenerateResultContent::Text(final_answer),
                        usage: total_usage,
                    });
                }
            }
        }
    }
}
