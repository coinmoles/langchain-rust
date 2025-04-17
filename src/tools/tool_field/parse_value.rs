use serde::de::Error as DeError;
use serde_json::{Map, Value};

use crate::utils::helper::to_unexpected;

use super::{
    ArrayField, BooleanField, IntegerField, NumberField, ObjectField, StringField, ToolField,
    ToolParameters,
};

pub(super) fn parse_tool_parameters_from_value(
    obj: &mut Map<String, Value>,
) -> Result<ToolParameters, serde_json::Error> {
    let properties = remove_object_properties(obj)?;
    let additional_properties = match obj.remove("additionalProperties") {
        Some(Value::Bool(additional_properties)) => Some(additional_properties),
        Some(Value::Null) => None,
        Some(other) => return Err(DeError::invalid_type(to_unexpected(&other), &"a boolean")),
        None => None,
    };
    let field = ToolParameters::new_full(properties, additional_properties);
    Ok(field)
}

fn parse_property_from_value(
    value: Value,
    name: String,
    required: bool,
) -> Result<Box<dyn ToolField>, serde_json::Error> {
    let Value::Object(mut obj) = value else {
        return Err(DeError::invalid_type(to_unexpected(&value), &"an object"));
    };

    let description = match obj.remove("description") {
        Some(Value::String(description)) => Some(description),
        Some(Value::Null) => None,
        Some(other) => return Err(DeError::invalid_type(to_unexpected(&other), &"a string")),
        None => None,
    };

    let r#type = match obj.remove("type").ok_or(DeError::missing_field("type"))? {
        Value::String(r#type) => r#type,
        Value::Array(types) => {
            if types.is_empty() {
                return Err(DeError::invalid_length(0, &"more than 1"));
            }

            let types = types
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s),
                    other => Err(DeError::invalid_type(to_unexpected(&other), &"string")),
                })
                .collect::<Result<Vec<_>, _>>()?;

            let types_str = format!("{:#?}", types);

            types
                .into_iter()
                .find(|t| {
                    matches!(
                        t.as_str(),
                        "string" | "integer" | "number" | "boolean" | "array" | "object"
                    )
                })
                .ok_or(DeError::invalid_value(
                    serde::de::Unexpected::Str(&types_str),
                    &"string, integer, number, boolean, array or object",
                ))?
        }
        other => return Err(DeError::invalid_type(to_unexpected(&other), &"a string")),
    };

    match r#type.as_str() {
        "string" => {
            let r#enum = remove_string_enum(&mut obj)?;
            let field = StringField::new_full(name, description, required, r#enum);
            Ok(Box::new(field))
        }
        "integer" => {
            let r#enum = remove_integer_enum(&mut obj)?;
            let field = IntegerField::new_full(name, description, required, r#enum);
            Ok(Box::new(field))
        }
        "number" => {
            let r#enum = get_number_enum(&mut obj)?;
            let field = NumberField::new_full(name, description, required, r#enum);
            Ok(Box::new(field))
        }
        "boolean" => {
            let r#enum = remove_boolean_enum(&mut obj)?;
            let field = BooleanField::new_full(name, description, required, r#enum);
            Ok(Box::new(field))
        }
        "array" => {
            let item = obj.remove("items").ok_or(DeError::missing_field("items"))?;
            let item = parse_property_from_value(item, name.clone(), required)?;
            let field = ArrayField::new_full(name, description, required, item);
            Ok(Box::new(field))
        }
        "object" => {
            let properties = remove_object_properties(&mut obj)?;
            let additional_properties = match obj.remove("additionalProperties") {
                Some(Value::Bool(additional_properties)) => Some(additional_properties),
                Some(Value::Null) => None,
                Some(other) => {
                    return Err(DeError::invalid_type(to_unexpected(&other), &"a boolean"))
                }
                None => None,
            };
            let field = ObjectField::new_full(
                name,
                description,
                required,
                properties,
                additional_properties,
            );
            Ok(Box::new(field))
        }
        _ => Err(DeError::invalid_value(
            serde::de::Unexpected::Str(&r#type),
            &"string, integer, number, boolean, array or object",
        )),
    }
}

fn remove_string_enum(
    obj: &mut Map<String, Value>,
) -> Result<Option<Vec<String>>, serde_json::Error> {
    match obj.remove("enum") {
        Some(Value::Array(array)) => {
            let enum_values: Vec<String> = array
                .into_iter()
                .map(|v| match v {
                    Value::String(s) => Ok(s),
                    other => Err(DeError::invalid_type(to_unexpected(&other), &"a string")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(enum_values))
        }
        Some(Value::Null) => Ok(None),
        Some(other) => Err(DeError::invalid_type(to_unexpected(&other), &"an array")),
        None => Ok(None),
    }
}

fn remove_integer_enum(
    obj: &mut Map<String, Value>,
) -> Result<Option<Vec<i64>>, serde_json::Error> {
    match obj.remove("enum") {
        Some(Value::Array(array)) => {
            let enum_values = array
                .into_iter()
                .map(|v| {
                    v.as_i64()
                        .ok_or(DeError::invalid_type(to_unexpected(&v), &"an integer"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(enum_values))
        }
        Some(Value::Null) => Ok(None),
        Some(other) => Err(DeError::invalid_type(to_unexpected(&other), &"an integer")),
        None => Ok(None),
    }
}

fn get_number_enum(obj: &mut Map<String, Value>) -> Result<Option<Vec<f64>>, serde_json::Error> {
    match obj.remove("enum") {
        Some(Value::Array(array)) => {
            let enum_values = array
                .into_iter()
                .map(|v| {
                    v.as_f64()
                        .ok_or(DeError::invalid_type(to_unexpected(&v), &"a number"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(enum_values))
        }
        Some(Value::Null) => Ok(None),
        Some(other) => Err(DeError::invalid_type(to_unexpected(&other), &"a number")),
        None => Ok(None),
    }
}

fn remove_boolean_enum(
    obj: &mut Map<String, Value>,
) -> Result<Option<Vec<bool>>, serde_json::Error> {
    match obj.remove("enum") {
        Some(Value::Array(array)) => {
            let enum_values = array
                .into_iter()
                .map(|v| {
                    v.as_bool()
                        .ok_or(DeError::invalid_type(to_unexpected(&v), &"a boolean"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(enum_values))
        }
        Some(Value::Null) => Ok(None),
        Some(other) => Err(DeError::invalid_type(to_unexpected(&other), &"a boolean")),
        None => Ok(None),
    }
}

fn remove_object_properties(
    obj: &mut Map<String, Value>,
) -> Result<Vec<Box<dyn ToolField>>, serde_json::Error> {
    let required = match obj.remove("required") {
        Some(Value::Array(array)) => array
            .into_iter()
            .map(|v| match v {
                Value::String(s) => Ok(s),
                other => Err(DeError::invalid_type(to_unexpected(&other), &"a string")),
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(other) => return Err(DeError::invalid_type(to_unexpected(&other), &"an array")),
        None => vec![],
    };

    let properties = match obj.remove("properties") {
        Some(Value::Object(properties)) => properties,
        Some(other) => return Err(DeError::invalid_type(to_unexpected(&other), &"an object")),
        None => Map::new(),
    };

    properties
        .into_iter()
        .map(|(name, value)| {
            let field_required = required.contains(&name);
            let field = parse_property_from_value(value, name, field_required)?;
            Ok(field)
        })
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_property() {
        let value = json!({
            "type": "string",
            "description": "A string field",
            "enum": ["value1", "value2"]
        });

        let field = parse_property_from_value(value.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("A string field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value);

        let value2 = json!({
            "type": "string",
            "description": "A string field",
        });

        let field = parse_property_from_value(value2.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("A string field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value2);
    }

    #[test]
    fn test_parse_number_property() {
        let value = json!({
            "type": "number",
            "description": "A number field",
            "enum": [1.0, 2.0, 3.0]
        });

        let field = parse_property_from_value(value.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("A number field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value);

        let value2 = json!({
            "type": "number",
        });
        let field = parse_property_from_value(value2.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), None);
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value2);
    }

    #[test]
    fn test_parse_integer_property() {
        let value = json!({
            "type": "integer",
            "description": "An integer field",
            "enum": [1, 2, 3]
        });

        let field = parse_property_from_value(value.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("An integer field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value);
    }

    #[test]
    fn test_parse_boolean_property() {
        let value = json!({
            "type": "boolean",
            "description": "A boolean field",
        });

        let field = parse_property_from_value(value.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("A boolean field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value);
    }

    #[test]
    fn test_array_property() {
        let value = json!({
            "type": "array",
            "description": "A string array field",
            "items": {
                "type": "string"
            }
        });

        let field = parse_property_from_value(value.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("A string array field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value);

        let value2 = json!({
            "type": "array",
            "description": "An integer array field",
            "items": {
                "type": "integer",
                "enum": [1, 2, 3]
            }
        });

        let field = parse_property_from_value(value2.clone(), "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), Some("An integer array field"));
        assert_eq!(field.required(), true);
        assert_eq!(field.to_openai_field(), value2);
    }

    #[test]
    fn test_object_property() {
        let value = json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The name of the tool"
                },
                "version": {
                    "type": "string",
                    "description": "The version of the tool"
                },
                "enabled": {
                    "type": "boolean",
                    "description": "Whether the tool is enabled"
                }
            },
            "required": ["name", "version"]
        });

        let field = parse_property_from_value(value, "test".to_string(), true).unwrap();
        assert_eq!(field.name(), "test");
        assert_eq!(field.description(), None);
        assert_eq!(field.required(), true);
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "The name of the tool"
                    },
                    "version": {
                        "type": "string",
                        "description": "The version of the tool"
                    },
                    "enabled": {
                        "type": "boolean",
                        "description": "Whether the tool is enabled"
                    }
                },
                "required": ["name", "version"],
                "additionalProperties": true
            })
        );
    }
}
