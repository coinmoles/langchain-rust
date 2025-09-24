use serde_json::Value;

use super::{balance_parenthesis, remove_multiline, remove_trailing_commas};

pub fn parse_partial_json(s: &str, strict: bool) -> Result<Value, serde_json::Error> {
    match serde_json::from_str::<Value>(s) {
        Ok(val) => return Ok(val),
        Err(e) if strict => return Err(e),
        Err(_) => (),
    }

    let multiline_removed = remove_multiline(s);
    if let Ok(val) = serde_json::from_str::<Value>(&multiline_removed) {
        return Ok(val);
    }

    // Step 2: Try parsing the cleaned version
    let comma_cleaned = remove_trailing_commas(&multiline_removed);
    if let Ok(val) = serde_json::from_str::<Value>(&comma_cleaned) {
        return Ok(val);
    }

    // Step 3: Attempt to balance braces/brackets
    let balanced = balance_parenthesis(&comma_cleaned);
    serde_json::from_str(&balanced)
}
