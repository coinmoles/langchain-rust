use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::schemas::ToolCall;

#[derive(Debug)]
pub enum AgentEvent {
    Action(Vec<ToolCall>),
    Finish(String),
}

impl<'de> Deserialize<'de> for AgentEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = Value::deserialize(deserializer)?;

        if let Some((id, name, arguments)) = take_action(&mut value) {
            Ok(AgentEvent::Action(vec![ToolCall {
                id,
                name,
                arguments,
            }]))
        } else if let Some(final_answer) = take_final_answer(&mut value) {
            Ok(AgentEvent::Finish(final_answer))
        } else {
            Err(serde::de::Error::custom(format!(
                "Invalid format: {}",
                value
            )))
        }
    }
}

/// Helper function to extract the action from the JSON value.
fn take_action(value: &mut Value) -> Option<(String, String, Value)> {
    // Do not want to early return since id can be missing
    let id = match value.get_mut("id").map(|v| v.take()) {
        Some(Value::String(id)) => id,
        _ => Uuid::new_v4().to_string(),
    };

    let action = match value.get_mut("action")?.take() {
        Value::String(action) => action,
        _ => return None,
    };

    let action_input = value.get_mut("action_input")?.take();

    Some((id, action, action_input))
}

/// Helper function to extract the final answer from the JSON value.
fn take_final_answer(value: &mut Value) -> Option<String> {
    let final_answer = match value.get_mut("final_answer")?.take() {
        Value::String(value) => value,
        other => serde_json::to_string_pretty(&other).unwrap_or_else(|_| other.to_string()),
    };

    Some(final_answer)
}
