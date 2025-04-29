use std::collections::HashSet;

use serde_json::Value;

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

pub fn is_malformed_event_str(text: &str, valid_keys: &[&[&str]]) -> bool {
    valid_keys
        .iter()
        .any(|valid| valid.iter().all(|key| text.contains(key)))
}
