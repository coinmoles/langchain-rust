use std::collections::HashSet;

use serde_json::Value;

/// Returns `true` if the JSON is likely a malformed event rather than a valid final answer.
///
/// Used to distinguish between malformed event outputs and valid JSON-formatted answers
/// to prevent incorrect classification in the event loop.
pub fn is_malformed_event(json: &Value, valid_keys: &[&[&str]]) -> bool {
    if let Some(obj) = json.as_object() {
        let keys = obj.keys().map(|s| s.as_str()).collect::<HashSet<_>>();

        if valid_keys
            .iter()
            .any(|valid| valid.iter().all(|key| keys.contains(key)))
        {
            return true;
        }
    }
    false
}

/// Returns `true` if the text is likely a malformed event rather than a valid final answer.
///
/// Used to distinguish between malformed event outputs and valid JSON-formatted answers
/// to prevent incorrect classification in the event loop.
pub fn is_malformed_event_str(text: &str, valid_keys: &[&[&str]]) -> bool {
    valid_keys.iter().any(|valid| {
        valid
            .iter()
            .all(|key| text.contains(&format!(r#""{key}""#)))
    })
}
