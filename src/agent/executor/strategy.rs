use std::collections::HashMap;

use async_trait::async_trait;

use crate::agent::Agent;
use crate::chain::{ChainError, InputCtor, OutputCtor};
use crate::schemas::{LLMOutput, Message};
use crate::tools::{FunctionTool, Tool, ToolOutput};

/// A pluggable policy that customizes **how an agent run is executed**.
///
/// `Strategy` lets you intercept and mutate every major phase of an
/// [`AgentExecutor`](crate::agent::AgentExecutor) run without changing the core loop:
///
/// # Methods:
/// 1. [`additional_tools`](Strategy::additional_tools) — inject extra tools to be used during the
///    execution.
/// 2. [`process_initial_messages`](Strategy::process_initial_messages) — inspect, validate, or
///    rewrite the initial messages.
/// 3. [`process_plan`](Strategy::process_plan) — inspect, validate, or rewrite the model-produced
///    [`LLMOutput`].
/// 4. [`process_tool_output`](Strategy::process_tool_output) — inspect, validate, or rewrite the
///    [`ToolOutput`].
/// 5. [`process_final_answer`](Strategy::process_final_answer) — inspect, validate, or rewrite the
///    final model answer.
/// 6. [`finalize`](Strategy::finalize) — produce a final output specific to the strategy.
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Type produced by [`finalize`]. Often used to return strategy-specific
    /// side artifacts (e.g., tag indices, telemetry, transcripts).
    type Output: Send + Sync;

    /// Additional tools to be used during this execution.
    ///
    /// The tools returned by this function will override the tools defined in the agent.
    fn additional_tools(&self) -> HashMap<&str, &Tool<'_>> {
        HashMap::new()
    }

    /// Inspect, validate, or rewrite the initial messages.
    ///
    /// Return the modified messages. Returning `Err` makes the execution fail immediately.
    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, ChainError> {
        Ok(messages)
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

    /// Inspect, validate, or rewrite the model-produced [`LLMOutput`].
    ///
    /// Return the modified plan. Returning `Err` makes the executor retry by making a new request
    /// to the model.
    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        Ok(plan)
    }

    /// Inspect, validate, or rewrite the [`ToolOutput`].
    ///
    /// Return the modified tool output. Returning `Err` makes the executor omit the corresponding
    /// tool call.
    async fn process_tool_output(
        &mut self,
        _call_id: &str,
        output: ToolOutput,
    ) -> Result<ToolOutput, ChainError> {
        Ok(output)
    }

    /// Inspect, validate, or rewrite the final model answer.
    ///
    /// Return the modified final answer. Returning `Err` makes the executor retry by making a new
    /// request to the model.
    async fn process_final_answer(&mut self, final_answer: String) -> Result<String, ChainError> {
        Ok(final_answer)
    }

    /// Produce a final output specific to the strategy.
    ///
    /// Returning `Err` makes the execution fail immediately. Avoid doing so unless it is strictly
    /// necessary.
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

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, ChainError> {
        if let Some(strategy) = self {
            strategy.process_initial_messages(messages).await
        } else {
            Ok(messages)
        }
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        if let Some(strategy) = self {
            strategy.process_plan(plan).await
        } else {
            Ok(plan)
        }
    }

    async fn process_tool_output(
        &mut self,
        call_id: &str,
        output: ToolOutput,
    ) -> Result<ToolOutput, ChainError> {
        if let Some(strategy) = self {
            strategy.process_tool_output(call_id, output).await
        } else {
            Ok(output)
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

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, ChainError> {
        let messages = self.0.process_initial_messages(messages).await?;
        self.1.process_initial_messages(messages).await
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        let plan = self.0.process_plan(plan).await?;
        self.1.process_plan(plan).await
    }

    async fn process_tool_output(
        &mut self,
        call_id: &str,
        output: ToolOutput,
    ) -> Result<ToolOutput, ChainError> {
        let output = self.0.process_tool_output(call_id, output).await?;
        self.1.process_tool_output(call_id, output).await
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

    async fn process_initial_messages(
        &mut self,
        messages: Vec<Message>,
    ) -> Result<Vec<Message>, ChainError> {
        let messages = self.0.process_initial_messages(messages).await?;
        let messages = self.1.process_initial_messages(messages).await?;
        self.2.process_initial_messages(messages).await
    }

    async fn process_plan(&mut self, plan: LLMOutput) -> Result<LLMOutput, ChainError> {
        let plan = self.0.process_plan(plan).await?;
        let plan = self.1.process_plan(plan).await?;
        self.2.process_plan(plan).await
    }

    async fn process_tool_output(
        &mut self,
        call_id: &str,
        output: ToolOutput,
    ) -> Result<ToolOutput, ChainError> {
        let output = self.0.process_tool_output(call_id, output).await?;
        let output = self.1.process_tool_output(call_id, output).await?;
        self.2.process_tool_output(call_id, output).await
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
