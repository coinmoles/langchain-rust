use std::collections::HashMap;
use std::fmt::Display;

use itertools::{Either, Itertools};
use tracing::{Instrument, info_span};

use crate::agent::{
    AgentError, AgentExecutor, AgentInput, AgentStep, DefaultStrategy, ExecutionOutput, Strategy,
};
use crate::chain::{ChainError, ChainOutput, InputCtor, OutputCtor};
use crate::llm::LLMOutput;
use crate::schemas::{IntoWithUsage, Message, Role, TokenUsage, ToolCall, ToolSpec, WithUsage};
use crate::tools::{FunctionTool, McpTool, Tool};
use crate::utils::helper::normalize_tool_name;

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
    pub input: AgentInput<I::Target<'input>>,
    /// The sequence of tool calls performed so far.
    pub steps: Vec<AgentStep>,
    /// Counts of how many times each tool has been invoked.
    pub use_counts: HashMap<String, usize>,
    /// The current number of consecutive failures.
    pub consecutive_fails: usize,
    /// Total token usage.
    pub total_usage: Option<TokenUsage>,
    /// Ephemeral tools
    pub mcp_functions: Option<HashMap<String, Box<dyn FunctionTool>>>,
    /// Tool spec for the run
    pub tool_spec: Option<ToolSpec>,
    /// Initial messages from the prompt
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
            input: AgentInput::new(input),
            steps: Vec::new(),
            use_counts: HashMap::new(),
            initial_messages: Vec::new(),
            consecutive_fails: 0,
            total_usage: None,
            mcp_functions: None,
            tool_spec: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Begin the execution.
    pub async fn start(mut self) -> Result<ExecutionOutput<'input, O, S>, ChainError> {
        let span = info_span!("agent", id = self.executor.agent.id());

        async move {
            self.input = self.strategy.prepare_input::<I>(self.input).await?;
            self.save_initial_messages()?;
            self.log_initial_messages()?;
            self.load_memory().await?;
            self.prepare_tools().await?;

            while !self.fail_limit_reached() {
                let Ok(plan) = self.plan_step().await else {
                    continue;
                };

                match plan {
                    LLMOutput::ToolCall(tool_calls) => self.handle_tool_calls(tool_calls).await,
                    LLMOutput::Text(final_answer) => match self.finalize(final_answer).await {
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

    fn save_initial_messages(&mut self) -> Result<(), ChainError> {
        self.initial_messages = self.executor.agent.get_prompt(&self.input)?.to_messages();
        Ok(())
    }

    fn log_initial_messages(&self) -> Result<(), ChainError> {
        if !log::log_enabled!(log::Level::Debug) {
            return Ok(());
        }

        for message in &self.initial_messages {
            log::debug!("{message}");
        }
        Ok(())
    }

    async fn prepare_tools(&mut self) -> Result<(), ChainError> {
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

        let (mcp_functions, spec) = if self.executor.agent.llm_chain.capabilities().native_mcp {
            (None, ToolSpec::from_tools(&functions, mcps))
        } else {
            let mcp_functions = McpTool::into_function_tools(mcps).await?;
            let all_functions = functions
                .into_iter()
                .chain(mcp_functions.values().map(|tool| tool.as_ref()))
                .collect::<Vec<_>>();
            let spec = ToolSpec::from_tools(&all_functions, Vec::new());
            (Some(mcp_functions), spec)
        };

        self.mcp_functions = mcp_functions;
        self.tool_spec = spec;
        Ok(())
    }

    pub async fn load_memory(&mut self) -> Result<(), ChainError> {
        if let Some(memory) = self.executor.memory.as_ref() {
            self.input.set_chat_history(memory.read().await.messages());
        }
        Ok(())
    }

    async fn plan_step(&mut self) -> Result<LLMOutput, ChainError> {
        // TODO: do not construct scratchpad every step.
        let scratchpad = self
            .steps
            .iter()
            .flat_map(|step| {
                [
                    Message::new_tool_call_message([step.tool_call.clone()]),
                    Message::new_tool_message(Some(step.tool_call.id.clone()), &step.result),
                ]
            })
            .collect::<Vec<_>>();
        self.input.set_agent_scratchpad(scratchpad);

        let plan = self
            .executor
            .agent
            .llm_chain
            .call_llm(&self.input, self.tool_spec.as_ref())
            .await
            .inspect_err(|e| failure!(self, "Failed to plan next step: {e}"))?;

        self.add_usage(plan.usage);
        let plan = self.strategy.process_plan(plan.content).await?;
        Ok(plan)
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

            let Ok((call, result)) = self
                .strategy
                .process_step(call, result)
                .await
                .inspect_err(|e| failure!(self, "Failed to process tool step: {e}"))
            else {
                return;
            };
            let step = AgentStep::new(call, result.data.to_string(), result.summary);
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
            let messages = self
                .initial_messages
                .into_iter()
                .chain(self.input.agent_scratchpad.unwrap_or_default())
                .chain([Message::new_ai_message(final_answer)])
                .filter(|m| m.role != Role::System)
                .collect();
            memory.write().await.add_messages(messages);
        }

        let WithUsage { content, usage } = answer.with_usage(self.total_usage);
        let extra_content = self
            .strategy
            .finalize()
            .await
            .map_err(FinalizeFailure::Abort)?;

        Ok(ExecutionOutput::new(content, extra_content, usage))
    }

    fn get_tool_with_use_count_check(&mut self, tool_name: &str) -> Option<&dyn FunctionTool> {
        let name = normalize_tool_name(tool_name);

        let tool = if let Some(tool) = self
            .mcp_functions
            .as_ref()
            .and_then(|funcs| funcs.get(&name))
        {
            tool.as_ref()
        } else if let Some(tool) = self.strategy.resolve_tool(&self.executor.agent, &name) {
            tool
        } else {
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
