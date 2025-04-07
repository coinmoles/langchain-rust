use serde_json::Value;

use super::{parse_value::parse_tool_parameters_from_value, ObjectField, ToolField};

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
