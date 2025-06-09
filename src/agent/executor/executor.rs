use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use indoc::formatdoc;
use tokio::sync::RwLock;

use crate::agent::{AgentError, AgentInput};
use crate::chain::Chain;
use crate::schemas::{ChainOutput, Ctor, InputCtor, IntoWithUsage, Prompt, WithUsage};
use crate::utils::helper::normalize_tool_name;
use crate::{
    agent::Agent,
    chain::ChainError,
    memory::Memory,
    schemas::{agent_plan::AgentEvent, AgentStep, TokenUsage},
};

use super::ExecutorOptions;

pub struct AgentExecutor<'a, I, O>
where
    I: InputCtor,
    O: Ctor,
    for<'b> I::Target<'b>: Display,
{
    agent: Box<dyn Agent<InputCtor = I, OutputCtor = O> + 'a>,
    memory: Option<Arc<RwLock<dyn Memory>>>,
    options: ExecutorOptions,
}

impl<'a, I, O> AgentExecutor<'a, I, O>
where
    I: InputCtor,
    O: Ctor,
    for<'b> I::Target<'b>: Display,
{
    pub fn from_agent(agent: impl Agent<InputCtor = I, OutputCtor = O> + 'a) -> Self {
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

    fn too_many_fails(&self, consecutive_fails: usize) -> Result<(), ChainError> {
        if self
            .options
            .max_consecutive_fails
            .is_some_and(|max_consecutive_fails| consecutive_fails >= max_consecutive_fails)
        {
            log::error!("Too many consecutive fails ({consecutive_fails} in a row), aborting");
            return Err(ChainError::AgentError("Too many consecutive fails".into()));
        }

        Ok(())
    }
}

#[async_trait]
impl<I, O> Chain for AgentExecutor<'_, I, O>
where
    I: InputCtor,
    O: Ctor,
    for<'b> I::Target<'b>: Display,
    for<'b> O::Target<'b>: ChainOutput<I::Target<'b>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        {
            let prompt = self.get_prompt(input.clone()).map_err(|e| {
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

        let human_message = input.to_string();
        let options = &self.options;

        let mut steps: Vec<AgentStep> = Vec::new();
        let mut use_counts: HashMap<String, usize> = HashMap::new();
        let mut consecutive_fails: usize = 0;
        let mut total_usage: Option<TokenUsage> = None;
        let mut input = AgentInput::new(input);

        if let Some(memory) = &self.memory {
            input.set_chat_history(memory.read().await.messages());
            // TODO: Possibly implement messages parsing
        }

        'step: loop {
            self.too_many_fails(consecutive_fails)?;

            let WithUsage { content, usage } = match self.agent.plan(&steps, &mut input).await {
                Ok(agent_event) => agent_event,
                Err(e) => {
                    consecutive_fails += 1;
                    log::warn!("Error: {} ({} consecutive fails)", e, consecutive_fails);
                    continue 'step;
                }
            };

            total_usage = TokenUsage::merge_options([&total_usage, &usage]);

            match content {
                AgentEvent::Action(tool_calls) => {
                    if options
                        .max_iterations
                        .is_some_and(|max_iterations| steps.len() >= max_iterations)
                    {
                        log::warn!(
                            "Max iteration ({}) reached, forcing final answer",
                            steps.len()
                        );
                        input.enable_ultimatum();
                        continue 'step;
                    }

                    for tool_call in tool_calls {
                        log::debug!("\nTool call:\n{tool_call}");

                        let tool_name = normalize_tool_name(&tool_call.name);
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
                                    formatdoc! {"
                                        Tool call failed: {e}
                                        If the error doesn't make sense to you, it means that the tool is broken. DO NOT use this tool again."
                                    }
                                }
                            }
                        };

                        log::debug!("\nTool {} result:\n{}", &tool_call.name, result);

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

                        memory.add_human_message(human_message);

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

                    let final_answer = O::Target::try_from_string(input.inner, final_answer)?;
                    return Ok(final_answer.with_usage(total_usage));
                }
            }
        }
    }

    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, ChainError> {
        self.agent.get_prompt(AgentInput::new(input))
    }
}
