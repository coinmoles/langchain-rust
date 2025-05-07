use serde_json::{Map, Value};
use uuid::Uuid;

use super::{balance_parenthesis, remove_multiline, remove_trailing_commas};

pub fn parse_partial_json(s: &str, strict: bool) -> Option<Value> {
    if let Ok(val) = serde_json::from_str::<Value>(s) {
        return Some(val);
    }

    if strict {
        return None;
    }

    let multiline_removed = remove_multiline(s);
    if let Ok(val) = serde_json::from_str::<Value>(&multiline_removed) {
        return Some(val);
    }

    // Step 2: Try parsing the cleaned version
    let comma_cleaned = remove_trailing_commas(&multiline_removed);
    if let Ok(val) = serde_json::from_str::<Value>(&comma_cleaned) {
        return Some(val);
    }

    // Step 3: Attempt to balance braces/brackets
    let balanced = balance_parenthesis(&comma_cleaned);
    serde_json::from_str(&balanced).ok()
}

/// Helper function to extract the action from the JSON value.
pub fn take_action(
    value: &mut Map<String, Value>,
    action_key: &str,
    action_input_key: &str,
) -> Option<(String, String, Value)> {
    // Do not want to early return since id can be missing
    let id = match value.remove("id") {
        Some(Value::String(id)) => id,
        _ => Uuid::new_v4().to_string(),
    };

    let Some(Value::String(action)) = value.remove(action_key) else {
        return None;
    };

    let action_input = value.remove(action_input_key).unwrap_or(Value::Null);

    Some((id, action, action_input))
}

/// Helper function to extract the final answer from the JSON value.
pub fn take_final_answer(value: &mut Map<String, Value>, final_answer_key: &str) -> Option<String> {
    let final_answer = match value.remove(final_answer_key)? {
        Value::String(value) => value,
        other => serde_json::to_string_pretty(&other).unwrap_or_else(|_| other.to_string()),
    };

    Some(final_answer)
}
