use async_trait::async_trait;

use crate::{
    agent::{Agent, AgentInput, AgentOutput, AgentStep},
    chain::{ChainError, InputCtor, OutputCtor},
    schemas::{ToolCall, WithUsage},
    tools::{ToolDyn, ToolOutput},
};

#[async_trait]
/// A pluggable policy that customizes **how an agent run is executed**.
///
/// `Strategy` lets you intercept and (optionally) mutate every major phase of an
/// [`AgentExecutor`] run without changing the core loop:
///
/// **Lifecycle (in order)**
/// 1. [`prepare_input`] — inject / normalize fields on the initial `AgentInput`.
/// 2. [`process_plan`] — validate or rewrite every model-produced `AgentOutput`.
/// 3. [`resolve_tool`] — optionally override which tool is called for a given name.
/// 4. [`build_step`] — turn each `(ToolCall, ToolOutput)` into an [`AgentStep`]
///    (e.g., reformatting, tagging, indexing).
/// 5. [`process_final_answer`] — validate/transform the final LLM answer before
///    converting it to `O::Target`.
/// 6. [`finalize`] — produce any strategy-specific artifact to return to the caller.
///
/// Additionally, you can implement [`agent_id`] to customize the log output.
///
/// All hooks have **no-op pass-through defaults** so you only override what you need.
///
/// ### Thread-safety
/// The trait is `Send + Sync` so a strategy can be shared across async tasks, but
/// each `ExecutionContext` holds a **distinct `Strategy` instance** (created with
/// `Default`) to keep per-run state without interior mutability.
///
/// ### Errors
/// All hooks return `ChainError`, allowing you to abort the run at any phase.
pub trait Strategy: Default + Send + Sync {
    /// Type produced by [`finalize`]. Often used to return strategy-specific
    /// side artifacts (e.g., tag indices, telemetry, transcripts).
    type Output;

    /// Unique identifier for the agent, used for logging and telemetry.
    fn agent_id(&self) -> Option<String> {
        None
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
        agent_input: AgentInput<I::Target<'input>>,
    ) -> Result<AgentInput<I::Target<'input>>, ChainError> {
        Ok(agent_input)
    }

    /// Resolve the concrete tool implementation to call for `tool_name`.
    ///
    /// Override this if you want to:
    /// - Substitute or shadow tools (e.g., for testing or routing).
    /// - Add indirections (aliases, fallbacks, version pinning, canary tools, ...).
    ///
    /// Default: delegates to `agent.get_tool(tool_name)`.
    fn resolve_tool<'tool, I: InputCtor, O: OutputCtor>(
        &'tool mut self,
        agent: &'tool dyn Agent<I, O>,
        tool_name: &str,
    ) -> Option<&'tool dyn ToolDyn>
    where
        Self: 'tool,
    {
        agent.get_tool(tool_name)
    }

    /// Inspect, validate, or rewrite the model-produced `AgentOutput` **each loop**.
    ///
    /// Typical uses:
    /// - Enforce JSON schema / tool-call structure.
    /// - Reject unsafe plans.
    /// - Add bookkeeping data to `usage`.
    ///
    /// Return the (possibly) modified plan. Returning `Err` makes the executor
    /// retry (until the fail limit) with the same context.
    async fn process_plan(
        &mut self,
        plan: WithUsage<AgentOutput>,
    ) -> Result<WithUsage<AgentOutput>, ChainError> {
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

#[derive(Default)]
pub struct DefaultStrategy;

#[async_trait]
impl Strategy for DefaultStrategy {
    type Output = ();

    async fn finalize(self) -> Result<Self::Output, ChainError> {
        Ok(())
    }
}
