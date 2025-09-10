use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(description = "The input for the tool")]
pub struct DefaultFunctionInput(pub String);

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmptyFunctionInput;

impl DefaultFunctionInput {
    pub fn new(input: impl Into<String>) -> Self {
        Self(input.into())
    }
}

impl From<String> for DefaultFunctionInput {
    fn from(input: String) -> Self {
        Self(input)
    }
}

impl From<&str> for DefaultFunctionInput {
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
        let schema = schema_for!(DefaultFunctionInput);
        let schema = serde_json::to_value(schema).unwrap();

        assert_eq!(
            schema,
            json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "DefaultFunctionInput",
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
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "title": "null",
                "type": "null",
            })
        )
    }
}
