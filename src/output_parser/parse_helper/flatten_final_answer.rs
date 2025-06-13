use serde_json::Value;

pub fn flatten_final_answer(mut final_answer: Value) -> Result<String, serde_json::Error> {
    while let Value::Object(mut map) = final_answer {
        if let Some(inner) = map.remove("final_answer") {
            final_answer = inner;
        } else {
            final_answer = Value::Object(map);
            break;
        }
    }

    match final_answer {
        Value::String(s) => Ok(s),
        other => serde_json::to_string_pretty(&other),
    }
}
