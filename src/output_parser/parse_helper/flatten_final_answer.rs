use serde_json::Value;

/// Flattens nested `final_answer` fields in a JSON value and returns the innermost value.
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

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case(
        json!({
            "final_answer": "This is the final answer"
        }),
        "This is the final answer"
    )]
    #[case(
        json!({
            "final_answer": {
                "final_answer": "This is the final answer"
            }
        }),
        "This is the final answer"
    )]
    #[case(
        json!({
            "final_answer": {
                "final_answer": {
                    "final_answer": "This is the final answer"
                }
            }
        }),
        "This is the final answer"
    )]
    fn test_flatten_final_answer(#[case] input: Value, #[case] expected: &str) {
        let result = super::flatten_final_answer(input).unwrap();
        assert_eq!(result, expected);
    }
}
