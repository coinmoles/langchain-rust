use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "The input for the tool")]
pub struct DefaultToolInput(pub String);

impl DefaultToolInput {
    pub fn new(input: impl Into<String>) -> Self {
        Self(input.into())
    }
}

impl From<String> for DefaultToolInput {
    fn from(input: String) -> Self {
        Self(input)
    }
}

impl From<&str> for DefaultToolInput {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}

#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_default_tool_input_schema() {
        let schema = schema_for!(DefaultToolInput);
        let schema = serde_json::to_value(schema).unwrap();

        assert_eq!(
            schema,
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": "DefaultToolInput",
                "description": "The input for the tool",
                "type": "string"
            })
        )
    }

    #[test]
    fn test_empty_tool_input_schema() {
        let schema = schema_for!(());
        let schema = serde_json::to_value(schema).unwrap();

        assert_eq!(
            schema,
            json!({
                "$schema": "http://json-schema.org/draft-07/schema#",
                "title": "Null",
                "type": "null",
            })
        )
    }
}
