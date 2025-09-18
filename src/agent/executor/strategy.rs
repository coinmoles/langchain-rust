use std::collections::HashMap;

use async_trait::async_trait;

use crate::agent::{Agent, AgentInput, AgentStep};
use crate::chain::{ChainError, InputCtor, OutputCtor};
use crate::llm::LLMOutput;
use crate::schemas::{ToolCall, ToolSpec};
use crate::tools::{FunctionTool, Tool, ToolOutput};

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
/// 1. [`additional_tools`] — inject extra tools to be used during this execution.
/// 2. [`prepare_input`] — inject / normalize fields on the initial `AgentInput`.
/// 3. [`process_plan`] — validate or rewrite every model-produced `AgentOutput`.
/// 4. [`build_step`] — turn each `(ToolCall, ToolOutput)` into an [`AgentStep`] (e.g.,
///    reformatting, tagging, indexing).
/// 5. [`process_final_answer`] — validate/transform the final LLM answer before converting it to
///    `O::Target`.
/// 6. [`finalize`] — produce any strategy-specific artifact to return to the caller.
///
/// All hooks have **no-op pass-through defaults** so you only override what you need.
#[async_trait]
pub trait Strategy: Default + Send + Sync {
    /// Type produced by [`finalize`]. Often used to return strategy-specific
    /// side artifacts (e.g., tag indices, telemetry, transcripts).
    type Output;

    /// Additional tools to be used during this execution.
    ///
    /// The tools returned here will override the tools defined in the agent.
    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        HashMap::new()
    }

    /// Prepare (augment / normalize) the initial `AgentInput` **before the first plan**.
    ///
    /// Typical uses:
    /// - Inject extra keys.
    /// - Pre-attach system hints or metadata.
    /// - Redact/normalize fields.
    ///
    /// Return the possibly modified `AgentInput`. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn prepare_input<'input, I: InputCtor>(
        &mut self,
        input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        Ok(input)
    }

    /// Resolve the concrete tool implementation to call for `tool_name`.
    ///
    /// It is recommended to leave the default implementation as-is, which checks (in order):
    /// 1. If the tool is defined in [`additional_tools`].
    /// 2. If the tool is defined in the agent’s static `tools`.
    /// 3. If the tool is defined in any of the agent’s `toolboxes`.
    ///
    /// Instead, override [`additional_tools`] to inject custom tools.
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

    /// Convert a `(ToolCall, ToolOutput)` pair into an [`AgentStep`] to be recorded.
    ///
    /// Typical uses:
    /// - Reformat or wrap tool outputs (e.g., XML/JSON tagging).
    /// - Maintain auxiliary indices/maps for later retrieval (store inside `self`).
    /// - Summarize or truncate large outputs.
    ///
    /// Return an `AgentStep` to append to the transcript. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn build_step(
        &mut self,
        call: ToolCall,
        output: ToolOutput,
    ) -> Result<AgentStep, ChainError> {
        let step = AgentStep::new(call, output.data.to_string(), output.summary);
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
