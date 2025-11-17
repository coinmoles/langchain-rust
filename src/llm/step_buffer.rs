use std::collections::{HashMap, HashSet};

use async_openai::types::ChatCompletionMessageToolCall;

use crate::schemas::ToolCall;
use crate::tools::{ToolData, ToolError, ToolOutput};

/// An helper struct for buffering tool calls during an llm session.
///
/// When a model generates multiple tool calls for a single step, it can be beneficial to exclude
/// failed tool calls from the chat history entirely. Instead of inserting the tool calls to chat
/// history and modifying them once the tool call fails, this struct can be used to preserve and
/// delay inserting the calls after they are completed.
#[derive(Default)]
#[repr(transparent)]
pub struct StepBuffer<C: CallId>(Option<BufferedStep<C>>);

impl<C: CallId> StepBuffer<C> {
    /// Constructs a new empty [`StepBuffer`].
    pub fn new() -> Self {
        Self(None)
    }

    /// Initializes a new step with the given thought and calls.
    pub fn init_step(&mut self, thought: Option<String>, calls: Vec<C>) {
        self.0 = Some(BufferedStep::new(thought, calls));
    }

    /// Pushes a result into the current step.
    pub fn push_result(
        &mut self,
        id: &str,
        tool_name: &str,
        result: Result<ToolOutput, ToolError>,
    ) {
        let Ok(output) = result else {
            return;
        };

        self.0
            .get_or_insert_default()
            .results
            .insert(id.to_string(), (tool_name.to_string(), output.data));
    }

    /// Attempts to take the current step for insertion.
    ///
    /// If none of the tool calls are resolved properly, an error is returned.
    pub fn try_take(&mut self) -> Result<Option<BufferedStep<C>>, ToolError> {
        let Some(BufferedStep {
            thought,
            calls,
            results,
        }) = self.0.take()
        else {
            return Ok(None);
        };
        if calls.is_empty() {
            return Ok(None);
        }

        let completed_call_ids = results.keys().map(String::as_str).collect::<HashSet<_>>();
        let completed_calls = calls
            .into_iter()
            .filter(|call| completed_call_ids.contains(call.id()))
            .collect::<Vec<_>>();

        if completed_calls.is_empty() {
            return Err(ToolError::AllToolsFailed);
        }
        Ok(Some(BufferedStep {
            thought,
            calls: completed_calls,
            results,
        }))
    }
}

/// The step buffered in [`StepBuffer`].
pub struct BufferedStep<C: CallId> {
    /// The "thought" portion of the original llm output.
    pub thought: Option<String>,

    /// The tool calls.
    pub calls: Vec<C>,

    /// The tool results stored in the HashMap. The key is the tool call id, and the value is a
    /// tuple of the tool name and the tool output.
    pub results: HashMap<String, (String, ToolData)>,
}

impl<C: CallId> BufferedStep<C> {
    /// Constructs a new [`StepBufferInner`]
    fn new(thought: Option<String>, calls: Vec<C>) -> Self {
        let len = calls.len();
        Self {
            thought,
            calls,
            results: HashMap::with_capacity(len),
        }
    }
}

impl<C: CallId> Default for BufferedStep<C> {
    fn default() -> Self {
        Self {
            thought: None,
            calls: Vec::new(),
            results: HashMap::new(),
        }
    }
}

/// A trait for identifying the unique identifier of a call.
pub trait CallId {
    /// Returns the unique identifier of the call.
    fn id(&self) -> &str;
}

impl CallId for ToolCall {
    fn id(&self) -> &str {
        &self.id
    }
}

impl CallId for ChatCompletionMessageToolCall {
    fn id(&self) -> &str {
        &self.id
    }
}
