use serde_json::{Map, Value};

use super::{parse_value::parse_tool_parameters_from_value, ObjectField, ToolField};

#[derive(Clone)]
pub struct ToolParameters(ObjectField);

impl ToolParameters {
    pub fn new(properties: Vec<Box<dyn ToolField>>, additional_properties: Option<bool>) -> Self {
        Self(ObjectField::new(
            "input",
            None,
            true,
            properties,
            additional_properties,
        ))
    }

    pub fn to_plain_description(&self) -> String {
        self.0.to_plain_description()
    }

    pub fn to_openai_field(&self) -> Value {
        self.0.to_openai_field()
    }

    pub fn properties_description(&self) -> String {
        self.0.properties_description()
    }
}

impl TryFrom<Value> for ToolParameters {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        parse_tool_parameters_from_value(value)
    }
}

impl TryFrom<Map<String, Value>> for ToolParameters {
    type Error = serde_json::Error;

    fn try_from(value: Map<String, Value>) -> Result<Self, Self::Error> {
        parse_tool_parameters_from_value(Value::Object(value))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_try_from_value() {
        let value = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "properties": {
                "a": {
                    "format": "int32",
                    "type": "integer",
                },
                "b": {
                    "format": "int32",
                    "type": "integer",
                },
            },
            "required": ["a", "b"],
            "title": "StructRequest",
            "type": "object",
        });

        let tool_parameters = super::ToolParameters::try_from(value).unwrap();
        println!("{:#?}", tool_parameters.to_openai_field());

        let value2 = json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "EmptyObject",
            "type": "object",
        });

        let tool_parameters2 = super::ToolParameters::try_from(value2).unwrap();
        println!("{:#?}", tool_parameters2.to_openai_field());
    }
}
