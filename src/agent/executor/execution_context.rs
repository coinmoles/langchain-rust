use std::collections::HashMap;
use std::fmt::Display;

use itertools::{Either, Itertools};
use tracing::instrument;

use crate::agent::{AgentError, AgentExecutor, DefaultStrategy, ExecutionOutput, Strategy};
use crate::chain::{ChainError, ChainOutput, InputCtor, OutputCtor};
use crate::llm::LlmSession;
use crate::schemas::{LLMEvent, Message, Role, TokenUsage, ToolCall, ToolSpec};
use crate::tools::{FunctionTool, Tool};
use crate::utils::helper::normalize_tool_name;

/// Helper macro to log the error and increment the consecutive error count.
macro_rules! failure {
    ($ctx:expr, $e:expr, $($args:tt)*) => {{
        $ctx.consecutive_fails += 1;
        log::warn!("{}: {} ({} consecutive fails)", ::core::format_args!($($args)*), $e, $ctx.consecutive_fails);
        Err($e.into())
    }};
    ($ctx:expr, $($args:tt)*) => {{
        $ctx.consecutive_fails += 1;
        log::warn!("{} ({} consecutive fails)", ::core::format_args!($($args)*), $ctx.consecutive_fails);
    }};
}

/// Helper enum to return the context on recoverable finalize failures.
enum FinalizeFailure<Ctx> {
    Retry(Ctx),
    Abort(ChainError),
}

/// Runtime context that owns mutable states during an [`AgentExecutor`] run.
pub struct ExecutionContext<'exec, 'tool, 'input, I, O, S = DefaultStrategy>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    /// Reference to the [`AgentExecutor`] driving this execution.
    executor: &'exec AgentExecutor<'tool, I, O>,
    /// The execution strategy for the execution.
    strategy: S,
    /// The input provided to this execution.
    input: I::Target<'input>,
    /// Counts of how many times each tool has been invoked.
    use_counts: HashMap<String, usize>,
    /// The current number of consecutive failures.
    consecutive_fails: usize,
    /// The current number of steps.
    step_count: usize,
    /// Total token usage.
    total_usage: Option<TokenUsage>,
    /// Initial messages sent to the LLM.
    initial_messages: Vec<Message>,
    _phantom: std::marker::PhantomData<O>,
}

impl<'exec, 'tool, 'input, I, O, S> ExecutionContext<'exec, 'tool, 'input, I, O, S>
where
    I: InputCtor,
    O: OutputCtor,
    S: Strategy,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    /// Constructs a new [`ExecutionContext`].
    #[must_use]
    pub fn new(
        executor: &'exec AgentExecutor<'tool, I, O>,
        input: I::Target<'input>,
        strategy: S,
    ) -> Self {
        Self {
            executor,
            strategy,
            input,
            use_counts: HashMap::new(),
            consecutive_fails: 0,
            step_count: 0,
            total_usage: None,
            initial_messages: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Begins the execution.
    #[instrument(name = "agent", level = "info", skip(self), fields(id = self.executor.agent.id()))]
    pub async fn start(mut self) -> Result<ExecutionOutput<'input, O, S>, ChainError> {
        let mut session = self.begin_session().await?;
        if let Some(memory) = &self.executor.memory {
            let memory = memory.read().await;
            session.load_memory(&*memory).await?;
        }

        while !self.fail_limit_reached() {
            let Ok(event) = self.advance_session(session.as_mut()).await else {
                continue;
            };
            match event {
                LLMEvent::ToolCall(tool_calls) => {
                    self.handle_tool_calls(session.as_mut(), tool_calls).await
                }
                LLMEvent::Text(final_answer) => match self.finalize(final_answer).await {
                    Ok(ok) => return Ok(ok),
                    Err(FinalizeFailure::Abort(e)) => return Err(e),
                    Err(FinalizeFailure::Retry(ctx)) => self = ctx,
                },
            }
            self.step_count += 1;
        }
        Err(AgentError::TooManyConsecutiveFails(self.consecutive_fails).into())
    }

    async fn prepare_messages(&mut self) -> Result<Vec<Message>, ChainError> {
        let input = &self.input;
        let messages = self.executor.agent.prompt.format(input)?.to_messages();
        let messages = self.strategy.process_initial_messages(messages).await?;

        self.initial_messages = messages.clone();
        if log::log_enabled!(log::Level::Debug) {
            for message in &messages {
                log::debug!("\n{message}");
            }
        }

        Ok(messages)
    }

    async fn begin_session(&mut self) -> Result<Box<dyn LlmSession + 'exec>, ChainError> {
        let messages = self.prepare_messages().await?;
        let spec = self.prepare_tools()?;
        let options = self.strategy.call_options().await;
        let session = self
            .executor
            .agent
            .llm
            .begin_session(messages, spec, options)
            .await?;
        Ok(session)
    }

    fn prepare_tools(&self) -> Result<Option<ToolSpec>, ChainError> {
        let (functions, mcps): (Vec<_>, Vec<_>) = self
            .executor
            .agent
            .tools
            .values()
            .chain(self.strategy.additional_tools().into_values())
            .partition_map(|tool| match tool {
                Tool::Function(func) => Either::Left(func.as_ref()),
                Tool::Mcp(mcp) => Either::Right(mcp.clone()),
            });
        let spec = ToolSpec::from_tools(&functions, mcps);
        Ok(spec)
    }

    async fn advance_session(
        &mut self,
        session: &mut dyn LlmSession,
    ) -> Result<LLMEvent, ChainError> {
        let output = match session.advance().await {
            Ok(output) => output,
            Err(e) => return failure!(self, e, "Failed to advance session"),
        };
        self.add_usage(output.usage);

        let output = self.strategy.process_plan(output.content).await?;
        if let Some(thought) = output.thought {
            log::debug!("\nLLM thought:\n{thought}");
        };

        Ok(output.event)
    }

    async fn handle_tool_calls(&mut self, session: &mut dyn LlmSession, tool_calls: Vec<ToolCall>) {
        if self.max_iterations_reached() {
            self.force_final_answer(session);
            return;
        }

        for call in tool_calls {
            log::debug!("\nTool call:\n{call}");

            let tool_name = normalize_tool_name(&call.name);
            let Some(tool) = self.get_tool_with_use_count_check(&tool_name) else {
                return failure!(self, "Tool '{tool_name}' not found");
            };

            let output = match tool.call(call.arguments.clone()).await {
                Ok(output) => {
                    log::debug!("\nTool {} result:\n{}", &call.name, output.data);
                    output
                }
                Err(e) => {
                    log::warn!("Tool {} error: {}", &call.name, e);
                    session.add_tool_result(&call.id, &call.name, Err(e));
                    continue;
                }
            };

            let output = match self.strategy.process_tool_output(&call.id, output).await {
                Ok(output) => output,
                Err(e) => return failure!(self, "Failed to process agent step: {e}"),
            };
            session.add_tool_result(&call.id, &call.name, Ok(output));
        }

        self.consecutive_fails = 0;
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

        let answer = match O::Target::from_text_and_input(self.input, final_answer.clone()) {
            Ok(answer) => answer,
            Err((returned_input, e)) => {
                // If the final answer cannot be constructed, `self.input.inner` is set again.
                self.input = returned_input;
                failure!(self, "Failed to construct output from final answer: {e}");
                return Err(FinalizeFailure::Retry(self));
            }
        };

        if let Some(memory) = &self.executor.memory {
            let messages = self
                .initial_messages
                .into_iter()
                .chain([Message::new_ai_message(final_answer)])
                .filter(|m| m.role != Role::System)
                .collect();
            memory.write().await.add_messages(messages);
        }

        let extra = self
            .strategy
            .finalize()
            .await
            .map_err(FinalizeFailure::Abort)?;

        Ok(ExecutionOutput::new(answer, extra, self.total_usage))
    }

    fn get_tool_with_use_count_check(&mut self, tool_name: &str) -> Option<&dyn FunctionTool> {
        let name = normalize_tool_name(tool_name);

        let Some(tool) = self.strategy.resolve_tool(&self.executor.agent, &name) else {
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
            .is_some_and(|max_iterations| self.step_count >= max_iterations)
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

    fn force_final_answer(&self, session: &mut dyn LlmSession) {
        log::warn!("Forcing final answer due to max iterations reached");
        session.force_final_answer()
    }
}
