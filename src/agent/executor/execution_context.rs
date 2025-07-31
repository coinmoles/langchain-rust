use std::{collections::HashMap, fmt::Display};

use tracing::{info_span, Instrument, Span};

use crate::{
    agent::{
        AgentError, AgentExecutor, AgentInput, AgentOutput, AgentStep, DefaultStrategy,
        ExecutionOutput, Strategy,
    },
    chain::{ChainError, ChainOutput, InputCtor, OutputCtor},
    schemas::{IntoWithUsage, TokenUsage, ToolCall, WithUsage},
    tools::ToolDyn,
    utils::helper::normalize_tool_name,
};

macro_rules! failure {
    ($ctx:expr, $($arg:tt)*) => {{
        $ctx.consecutive_fails += 1;
        log::warn!("{} ({} consecutive fails)", ::core::format_args!($($arg)*), $ctx.consecutive_fails);
    }};
}

enum FinalizeFailure<Ctx> {
    Retry(Ctx),
    Abort(ChainError),
}

/// Runtime context that owns all mutable state during an [`AgentExecutor`] run.
///
/// * `steps`              - transcript of executed tool calls so far
/// * `use_counts`         - per-tool invocation counters
/// * `consecutive_fails`  - back-off & abort logic
/// * `total_usage`        - cumulative token accounting
pub struct ExecutionContext<'exec, 'agent, 'input, I, O, S = DefaultStrategy>
where
    I: InputCtor,
    O: OutputCtor,
    S: Strategy,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    executor: &'exec AgentExecutor<'agent, I, O>,
    strategy: S,
    input: AgentInput<I::Target<'input>>,
    steps: Vec<AgentStep>,
    use_counts: HashMap<String, usize>,
    consecutive_fails: usize,
    total_usage: Option<TokenUsage>,
}

impl<'exec, 'agent, 'input, I, O, S> ExecutionContext<'exec, 'agent, 'input, I, O, S>
where
    I: InputCtor,
    O: OutputCtor,
    S: Strategy,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    #[must_use]
    pub fn new(
        executor: &'exec AgentExecutor<'agent, I, O>,
        input: I::Target<'input>,
        strategy: S,
    ) -> Self {
        Self {
            executor,
            input: AgentInput::new(input),
            steps: Vec::new(),
            use_counts: HashMap::new(),
            consecutive_fails: 0,
            total_usage: None,
            strategy,
        }
    }

    pub fn with_strategy(mut self, strategy: S) -> Self {
        self.strategy = strategy;
        self
    }

    /// Entry point – iteratively plan / execute tool actions until the agent
    /// produces a valid final answer that can be transformed into `O`.
    pub async fn start(mut self) -> Result<ExecutionOutput<'input, O, S>, ChainError> {
        let span = if let Some(id) = self.strategy.agent_id() {
            info_span!("agent_execution", %id)
        } else {
            Span::none()
        };

        async move {
            self.load_memory().await;
            self.input = self.strategy.prepare_input::<I>(self.input).await?;
            self.log_initial_prompt()?;

            while !self.fail_limit_reached() {
                let Ok(plan) = self.plan_step().await else {
                    continue;
                };

                match plan {
                    AgentOutput::Action(tool_calls) => self.handle_tool_calls(tool_calls).await,
                    AgentOutput::Finish(final_answer) => match self.finalize(final_answer).await {
                        Ok(ok) => return Ok(ok),
                        Err(FinalizeFailure::Abort(e)) => return Err(e),
                        Err(FinalizeFailure::Retry(new_context)) => self = new_context,
                    },
                }
            }
            Err(AgentError::TooManyConsecutiveFails(self.consecutive_fails).into())
        }
        .instrument(span)
        .await
    }

    fn log_initial_prompt(&self) -> Result<(), ChainError> {
        if !log::log_enabled!(log::Level::Debug) {
            return Ok(());
        }

        for message in self.executor.agent.get_prompt(&self.input)?.to_messages() {
            log::debug!(
                "{}:\n{}",
                message.message_type.to_string().to_uppercase(),
                message.content
            );
        }
        Ok(())
    }

    async fn load_memory(&mut self) {
        if let Some(memory) = &self.executor.memory {
            self.input.set_chat_history(memory.read().await.messages());
        }
    }

    async fn plan_step(&mut self) -> Result<AgentOutput, ChainError> {
        let scratchpad = self
            .executor
            .agent
            .construct_scratchpad(&self.steps)
            .await?;
        self.input.set_agent_scratchpad(scratchpad);

        let plan = self
            .executor
            .agent
            .plan(&self.input)
            .await
            .inspect_err(|e| failure!(self, "Failed to plan next step: {e}"))?;

        let plan = self.strategy.process_plan(plan).await?;
        self.add_usage(plan.usage);
        Ok(plan.content)
    }

    async fn handle_tool_calls(&mut self, tool_calls: Vec<ToolCall>) {
        if self.max_iterations_reached() {
            self.force_final_answer();
            return;
        }

        for call in tool_calls {
            log::debug!("\nTool call:\n{call}");
            let tool_name = normalize_tool_name(&call.name);

            let Some(tool) = self.get_tool_with_use_count_check(&tool_name) else {
                return;
            };

            let Ok(result) = tool
                .call(call.arguments.clone())
                .await
                .inspect_err(|e| failure!(self, "Tool '{tool_name}' error: {e}"))
            else {
                return;
            };

            log::trace!("\nTool {} raw result:\n{}", &call.name, result.data);

            let Ok(step) = self
                .strategy
                .build_step(call, result)
                .await
                .inspect_err(|e| failure!(self, "Failed to construct tool step: {e}"))
            else {
                return;
            };
            log::debug!("\nTool {} result:\n{}", &step.tool_call.name, step.result);
            self.steps.push(step);
            self.consecutive_fails = 0;
        }
    }

    async fn finalize(
        mut self,
        final_answer: String,
    ) -> Result<ExecutionOutput<'input, O, S>, FinalizeFailure<Self>> {
        let Ok(final_answer) = self.strategy.process_final_answer(final_answer).await else {
            failure!(self, "Failed to construct final answer");
            return Err(FinalizeFailure::Retry(self));
        };

        log::debug!("\nAgent finished with result:\n{final_answer}");

        let human_message = self.input.inner.to_string();
        // `self.input.inner` is moved here, this cannot be done in a separate method which receives `&self`.
        let answer = match O::Target::from_text_and_input(self.input.inner, final_answer.clone()) {
            Ok(answer) => answer,
            Err((returned_input, e)) => {
                // If the final answer cannot be constructed, `self.input.inner` is set again.
                self.input.inner = returned_input;
                failure!(self, "Failed to construct output from final answer: {e}");
                return Err(FinalizeFailure::Retry(self));
            }
        };

        if let Some(memory) = &self.executor.memory {
            memory
                .write()
                .await
                .update(human_message, self.steps, final_answer);
        }

        let WithUsage { content, usage } = answer.with_usage(self.total_usage);
        let extra_content = self
            .strategy
            .finalize()
            .await
            .map_err(FinalizeFailure::Abort)?;

        Ok(ExecutionOutput::new(content, extra_content, usage))
    }

    fn get_tool_with_use_count_check(&mut self, tool_name: &str) -> Option<&dyn ToolDyn> {
        let Some(tool) = self
            .strategy
            .resolve_tool(self.executor.agent.as_ref(), tool_name)
        else {
            failure!(self, "Failed to fetch tool '{tool_name}'");
            return None;
        };
        if let Some(limit) = tool.usage_limit() {
            let count = self.use_counts.entry(tool_name.to_string()).or_default();
            *count += 1;
            if *count > limit {
                failure!(self, "Tool '{tool_name}' usage limit reached ({limit})");
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

    fn add_usage(&mut self, usage: Option<TokenUsage>) {
        self.total_usage = TokenUsage::merge_options([&self.total_usage, &usage]);
    }

    fn force_final_answer(&mut self) {
        log::warn!("Forcing final answer due to max iterations reached");
        self.input.enable_ultimatum();
    }
}
