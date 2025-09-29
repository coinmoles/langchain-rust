use std::collections::HashMap;

use async_trait::async_trait;

use crate::agent::{Agent, AgentInput, AgentStep};
use crate::chain::{ChainError, InputCtor, OutputCtor};
use crate::schemas::{LLMOutput, Message, ToolSpec};
use crate::tools::{FunctionTool, Tool};

/// The tools resolved for the current execution.
pub struct ResolvedTools {
    /// The mapping from the tool name to their implementation. Whether it be local function tools
    /// or MCP tools that are treated as function tools.
    pub mcp_functions: Option<HashMap<String, Box<dyn FunctionTool>>>,
    /// The tool specification to be sent to the LLM.
    pub spec: Option<ToolSpec>,
}

/// A pluggable policy that customizes **how an agent run is executed**.
///
/// `Strategy` lets you intercept and (optionally) mutate every major phase of an
/// [`AgentExecutor`](crate::agent::AgentExecutor) run without changing the core loop:
///
/// **Lifecycle (in order)**
/// 1. [`additional_tools`](Strategy::additional_tools) — inject extra tools to be used during this
///    execution.
/// 2. [`prepare_input`](Strategy::prepare_input) — inject / normalize fields on the initial
///    [`AgentInput`].
/// 3. [`process_plan`](Strategy::process_plan) — validate or rewrite every model-produced
///    [`LLMOutput`].
/// 4. [`process_step`](Strategy::process_step) — validate or rewrite every [`AgentStep`] before
///    appending it to the transcript.
/// 5. [`process_final_answer`](Strategy::process_final_answer) — validate/transform the final LLM
///    answer before converting it to `O::Target`.
/// 6. [`finalize`](Strategy::finalize) — produce any strategy-specific artifact to return to the
///    caller.
///
/// All hooks have **no-op pass-through defaults** so you only override what you need.
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Type produced by [`finalize`]. Often used to return strategy-specific
    /// side artifacts (e.g., tag indices, telemetry, transcripts).
    type Output: Send + Sync;

    /// Additional tools to be used during this execution.
    ///
    /// The tools returned here will override the tools defined in the agent.
    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        HashMap::new()
    }

    /// Prepare (augment / normalize) the initial [`AgentInput`] **before the first plan**.
    ///
    /// Typical uses:
    /// - Inject extra keys.
    /// - Pre-attach system hints or metadata.
    /// - Redact/normalize fields.
    ///
    /// Return the possibly modified [`AgentInput`]. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn prepare_input<'input, I: InputCtor>(
        &mut self,
        input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        Ok(input)
    }

    /// Scan the initial messages from memory before starting the execution.
    ///
    /// Typical uses:
    /// - Analyze the initial messages to set up context or state.
    ///
    /// Return `Ok(())` if successful. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn scan_initial_messages(&mut self, _messages: &[Message]) -> Result<(), ChainError> {
        Ok(())
    }

    /// Resolve the concrete tool implementation to call for `tool_name`.
    ///
    /// It is recommended to leave the default implementation as-is, which checks (in order):
    /// 1. If the tool is defined in [`additional_tools`](Strategy::additional_tools).
    /// 2. If the tool is defined in the agent’s static [`tools`](Agent::tools).
    /// 3. If the tool is defined in any of the agent’s [`toolboxes`](Agent::toolboxes).
    ///
    /// Instead, override [`additional_tools`](Strategy::additional_tools) to inject custom tools.
    fn resolve_tool<'tool, I: InputCtor, O: OutputCtor>(
        &'tool mut self,
        agent: &'tool Agent<I, O>,
        tool_name: &str,
    ) -> Option<&'tool dyn FunctionTool>
    where
        Self: 'tool,
    {
        if let Some(Tool::Function(func)) = self.additional_tools().get(tool_name) {
            Some(func.as_ref())
        } else if let Some(Tool::Function(func)) = agent.tools.get(tool_name) {
            Some(func.as_ref())
        } else if let Some(func) = agent
            .toolboxes
            .iter()
            .find_map(|toolbox| toolbox.get_tool(tool_name))
        {
            Some(func)
        } else {
            None
        }
    }

    /// Inspect, validate, or rewrite the model-produced [`LLMOutput`] **each loop**.
    ///
    /// Typical uses:
    /// - Reject unsafe plans.
    /// - Add bookkeeping data to `usage`.
    ///
    /// Return the (possibly) modified plan. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        Ok(plan)
    }

    /// Processes the tool call and its output ([`AgentStep`]) **after each tool execution**.
    ///
    /// Typical uses:
    /// - Reformat or wrap tool outputs (e.g., XML/JSON tagging).
    /// - Maintain auxiliary indices/maps for later retrieval (store inside `self`).
    /// - Summarize or truncate large outputs.
    ///
    /// Return an [`AgentStep`] to append to the transcript. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn process_step(&mut self, step: AgentStep) -> Result<AgentStep, ChainError> {
        Ok(step)
    }

    /// Validate / transform the final model answer **before** it is converted into `O::Target`.
    ///
    /// Typical uses:
    /// - Guardrails (structure, safety, hallucination checks).
    /// - Post-processing (e.g., fix malformed JSON, inject references).
    ///
    /// Return the (possibly) modified final answer string. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn process_final_answer(&mut self, final_answer: String) -> Result<String, ChainError> {
        Ok(final_answer)
    }

    /// Final hook called **once the run successfully completes**.
    ///
    /// Use this to emit any accumulated per-run artifact (indexes, telemetry, logs, …).
    ///
    /// Returning `Err` aborts the run **after** the model produced a valid answer, so only do
    /// this if you strictly need to guarantee the auxiliary artifact; otherwise prefer logging.
    async fn finalize(self) -> Result<Self::Output, ChainError>;
}

/// The default strategy without any special behavior.
#[derive(Default)]
pub struct DefaultStrategy;

#[async_trait]
impl Strategy for DefaultStrategy {
    type Output = ();

    async fn finalize(self) -> Result<Self::Output, ChainError> {
        Ok(())
    }
}

#[async_trait]
impl<S> Strategy for Option<S>
where
    S: Strategy,
{
    type Output = Option<S::Output>;

    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        if let Some(strategy) = self {
            strategy.additional_tools()
        } else {
            HashMap::new()
        }
    }

    async fn prepare_input<'input, I: InputCtor>(
        &mut self,
        input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        if let Some(strategy) = self {
            strategy.prepare_input::<'_, '_, '_, I>(input).await
        } else {
            Ok(input)
        }
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        if let Some(strategy) = self {
            strategy.process_plan(plan).await
        } else {
            Ok(plan)
        }
    }

    async fn process_step(&mut self, step: AgentStep) -> Result<AgentStep, ChainError> {
        if let Some(strategy) = self {
            strategy.process_step(step).await
        } else {
            Ok(step)
        }
    }

    async fn process_final_answer(&mut self, final_answer: String) -> Result<String, ChainError> {
        if let Some(strategy) = self {
            strategy.process_final_answer(final_answer).await
        } else {
            Ok(final_answer)
        }
    }

    async fn finalize(self) -> Result<Self::Output, ChainError> {
        if let Some(strategy) = self {
            let output = strategy.finalize().await?;
            Ok(Some(output))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl<S1, S2> Strategy for (S1, S2)
where
    S1: Strategy,
    S2: Strategy,
{
    type Output = (S1::Output, S2::Output);

    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        self.0
            .additional_tools()
            .into_iter()
            .chain(self.1.additional_tools())
            .collect()
    }
    async fn prepare_input<'input, I: InputCtor>(
        &mut self,
        input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        let input = self.0.prepare_input::<'_, '_, '_, I>(input).await?;
        self.1.prepare_input::<'_, '_, '_, I>(input).await
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        let plan = self.0.process_plan(plan).await?;
        self.1.process_plan(plan).await
    }

    async fn process_step(&mut self, step: AgentStep) -> Result<AgentStep, ChainError> {
        let step = self.0.process_step(step).await?;
        self.1.process_step(step).await
    }

    async fn process_final_answer(&mut self, final_answer: String) -> Result<String, ChainError> {
        let answer = self.0.process_final_answer(final_answer).await?;
        self.1.process_final_answer(answer).await
    }

    async fn finalize(self) -> Result<Self::Output, ChainError> {
        let output = self.0.finalize().await?;
        let output2 = self.1.finalize().await?;
        Ok((output, output2))
    }
}

#[async_trait]
impl<S1, S2, S3> Strategy for (S1, S2, S3)
where
    S1: Strategy,
    S2: Strategy,
    S3: Strategy,
{
    type Output = (S1::Output, S2::Output, S3::Output);

    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        self.0
            .additional_tools()
            .into_iter()
            .chain(self.1.additional_tools())
            .chain(self.2.additional_tools())
            .collect()
    }

    async fn prepare_input<'input, I: InputCtor>(
        &mut self,
        input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        let input = self.0.prepare_input::<'_, '_, '_, I>(input).await?;
        let input = self.1.prepare_input::<'_, '_, '_, I>(input).await?;
        self.2.prepare_input::<'_, '_, '_, I>(input).await
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        let plan = self.0.process_plan(plan).await?;
        let plan = self.1.process_plan(plan).await?;
        self.2.process_plan(plan).await
    }

    async fn process_step(&mut self, step: AgentStep) -> Result<AgentStep, ChainError> {
        let step = self.0.process_step(step).await?;
        let step = self.1.process_step(step).await?;
        self.2.process_step(step).await
    }

    async fn process_final_answer(&mut self, final_answer: String) -> Result<String, ChainError> {
        let answer = self.0.process_final_answer(final_answer).await?;
        let answer = self.1.process_final_answer(answer).await?;
        self.2.process_final_answer(answer).await
    }

    async fn finalize(self) -> Result<Self::Output, ChainError> {
        let output = self.0.finalize().await?;
        let output2 = self.1.finalize().await?;
        let output3 = self.2.finalize().await?;
        Ok((output, output2, output3))
    }
}
