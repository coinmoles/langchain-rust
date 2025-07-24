use std::{collections::HashMap, fmt::Display};

use crate::{
    agent::{AgentError, AgentExecutor, AgentInput, AgentOutput, AgentStep},
    chain::{ChainError, ChainOutput, InputCtor, OutputCtor},
    schemas::{IntoWithUsage, TokenUsage, ToolCall, WithUsage},
    tools::ToolDyn,
    utils::helper::normalize_tool_name,
};

/// Runtime context that owns all mutable state during an [`AgentExecutor`] run.
///
/// * `steps` - transcript of executed tool calls so far
/// * `use_counts` - per-tool invocation counters (enforced via the tool’s `usage_limit()`)
/// * `fails_in_a_row` - consecutive planning / execution failures for back-off & abort logic
/// * `total_usage` - cumulative token accounting (merged from the LLM and tools)
pub struct ExecutionContext<'exec, 'agent, 'input, I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    executor: &'exec AgentExecutor<'agent, I, O>,
    input: AgentInput<I::Target<'input>>,
    steps: Vec<AgentStep>,
    use_counts: HashMap<String, usize>,
    consecutive_fails: usize,
    total_usage: Option<TokenUsage>,
}

impl<'exec, 'agent, 'input, I, O> ExecutionContext<'exec, 'agent, 'input, I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn new(executor: &'exec AgentExecutor<'agent, I, O>, input: I::Target<'input>) -> Self {
        Self {
            executor,
            input: AgentInput::new(input),
            steps: Vec::new(),
            use_counts: HashMap::new(),
            consecutive_fails: 0,
            total_usage: None,
        }
    }

    /// Entry point – iteratively plan / execute tool actions until the agent
    /// produces a valid final answer that can be transformed into `O`.
    pub async fn start(mut self) -> Result<WithUsage<O::Target<'input>>, ChainError> {
        self.log_initial_prompt()?;
        self.load_memory().await;

        while !self.fail_limit_reached() {
            let Ok(plan) = self.plan_next_step().await else {
                continue;
            };

            match plan {
                AgentOutput::Action(tool_calls) => self.execute_tool_calls(tool_calls).await,
                // This match arm cannot be extracted into a separate method because of ownership issue involving `self.input.inner`.
                AgentOutput::Finish(final_answer) => {
                    if !self.is_final_answer_valid(&final_answer) {
                        self.bump_failure(&format!(
                            "Final answer validation failed: {final_answer}"
                        ));
                        continue;
                    }

                    log::debug!("\nAgent finished with result:\n{final_answer}");

                    let human_message = self.input.inner.to_string();
                    // `self.input.inner` is moved here, this cannot be done in a separate method which receives `&self`.
                    let answer = match O::Target::construct_from_text_and_input(
                        self.input.inner,
                        final_answer.clone(),
                    ) {
                        Ok(answer) => answer,
                        Err((returned_input, e)) => {
                            // If the final answer cannot be constructed, `self.input.inner` is set again.
                            self.input.inner = returned_input;
                            self.bump_failure(&format!(
                                "Failed to construct output from final answer: {e}"
                            ));
                            continue;
                        }
                    };

                    if let Some(memory) = &self.executor.memory {
                        memory
                            .write()
                            .await
                            .update(human_message, self.steps, final_answer);
                    }

                    return Ok(answer.with_usage(self.total_usage));
                }
            }
        }

        Err(AgentError::TooManyConsecutiveFails(self.consecutive_fails).into())
    }

    fn log_initial_prompt(&self) -> Result<(), ChainError> {
        if log::log_enabled!(log::Level::Debug) {
            for message in self.executor.agent.get_prompt(&self.input)?.to_messages() {
                log::debug!(
                    "\n{}:\n{}",
                    message.message_type.to_string().to_uppercase(),
                    message.content
                );
            }
        }
        Ok(())
    }

    async fn load_memory(&mut self) {
        if let Some(memory) = &self.executor.memory {
            self.input.set_chat_history(memory.read().await.messages());
        }
    }

    async fn plan_next_step(&mut self) -> Result<AgentOutput, ChainError> {
        match self.executor.agent.plan(&self.steps, &mut self.input).await {
            Ok(plan) => {
                self.add_usage(plan.usage);
                Ok(plan.content)
            }
            Err(e) => {
                self.bump_failure(&format!("Failed to plan next step: {e}"));
                Err(e.into())
            }
        }
    }

    async fn execute_tool_calls(&mut self, tool_calls: Vec<ToolCall>) {
        if self.max_iterations_reached() {
            self.force_final_answer();
            return;
        }

        for call in tool_calls {
            log::debug!("\nTool call:\n{call}");
            let tool_name = normalize_tool_name(&call.name);

            let Some(tool) = self.get_tool_with_use_count_check(&tool_name) else {
                continue;
            };

            match tool.call(call.arguments.clone()).await {
                Ok(result) => {
                    log::debug!("\nTool {} result:\n{}", &call.name, result.data);
                    let step = AgentStep::new(call, result.data.to_string(), result.summary);
                    self.steps.push(step);
                    self.consecutive_fails = 0;
                }
                Err(e) => {
                    self.bump_failure(&format!("Tool '{tool_name}' error: {e}"));
                    return;
                }
            };
        }
    }

    fn get_tool_with_use_count_check(&mut self, tool_name: &str) -> Option<&dyn ToolDyn> {
        let Some(tool) = self.executor.agent.get_tool(tool_name) else {
            self.bump_failure(&format!("Tried to use nonexistent tool '{tool_name}'"));
            return None;
        };
        if let Some(usage_limit) = tool.usage_limit() {
            let count = self.use_counts.entry(tool_name.to_string()).or_insert(0);
            *count += 1;
            if *count > usage_limit {
                self.bump_failure(&format!(
                    "Tool '{tool_name}' usage limit reached ({usage_limit})"
                ));
                return None;
            }
        }
        Some(tool)
    }

    fn max_iterations_reached(&self) -> bool {
        self.executor
            .options
            .max_iterations
            .is_some_and(|max_iterations| self.steps.len() >= max_iterations)
    }

    fn fail_limit_reached(&self) -> bool {
        self.executor
            .options
            .max_consecutive_fails
            .is_some_and(|max_consecutive_fails| self.consecutive_fails >= max_consecutive_fails)
    }

    fn is_final_answer_valid(&self, final_answer: &str) -> bool {
        if let Some(validator) = &self.executor.options.final_answer_validator {
            validator.validate_final_answer(final_answer, &self.steps)
        } else {
            true
        }
    }

    fn add_usage(&mut self, usage: Option<TokenUsage>) {
        self.total_usage = TokenUsage::merge_options([&self.total_usage, &usage]);
    }

    fn bump_failure(&mut self, message: &str) {
        self.consecutive_fails += 1;
        log::warn!("{message} ({} consecutive fails)", self.consecutive_fails);
    }

    fn force_final_answer(&mut self) {
        log::warn!("Forcing final answer due to max iterations reached");
        self.input.enable_ultimatum();
    }
}
