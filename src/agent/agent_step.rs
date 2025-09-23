use crate::schemas::{Message, ToolCall};
use crate::tools::ToolData;

/// A single step taken by the agent, including the thought process and actions taken.
#[derive(Clone, Debug)]
pub struct AgentStep {
    pub thought: Option<String>,
    pub actions: Vec<AgentAction>,
}

/// A single action taken by the agent, including the tool call and its result.
#[derive(Clone, Debug)]
pub struct AgentAction {
    /// The tool call that caused this action.
    pub tool_call: ToolCall,
    /// The result of the tool call.
    pub output: ToolData,
    /// An optional summary of the step, providing additional context or information.
    pub summary: Option<String>,
}

impl AgentStep {
    /// Creates a new `AgentStep` with the specified thought, actions.
    pub fn new(thought: Option<String>, actions: Vec<AgentAction>) -> Self {
        Self { thought, actions }
    }

    pub fn into_messages(self) -> impl IntoIterator<Item = Message> {
        let mut tool_calls = Vec::with_capacity(self.actions.len());
        let mut result_msgs = Vec::with_capacity(self.actions.len());

        for action in self.actions {
            let id = action.tool_call.id.clone();
            let msg = Message::new_tool_message(Some(id), action.output.to_string());
            result_msgs.push(msg);
            tool_calls.push(action.tool_call);
        }

        let tool_call_msg = Message::new_tool_call_message(self.thought, tool_calls);
        std::iter::once(tool_call_msg).chain(result_msgs)
    }
}

impl AgentAction {
    /// Creates a new `AgentAction` with the specified tool call, result, and summary.
    pub fn new(tool_call: ToolCall, result: ToolData, summary: Option<String>) -> Self {
        Self {
            tool_call,
            output: result,
            summary,
        }
    }
}
