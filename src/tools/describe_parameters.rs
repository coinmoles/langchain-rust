use std::collections::BTreeMap;

use gix::hashtable::hash_set::HashSet;
use indoc::formatdoc;
use regex::Regex;
use schemars::Schema;
use serde_json::Value;

use crate::utils::helper::add_indent;

pub fn describe_parameters(parameters: &Schema) -> Result<String, String> {
    let definitions = collect_definitions(parameters)?;

    describe_schema(parameters, true, &definitions, 0)
}

fn collect_definitions(root: &Schema) -> Result<BTreeMap<&str, &Schema>, String> {
    let mut out = BTreeMap::new();

    // Prefer $defs (2019-09 / 2020-12), but also support older "definitions"
    for key in ["$defs", "definitions"] {
        let Some(Value::Object(defs)) = root.get(key) else {
            continue;
        };
        for (k, v) in defs {
            let def = v
                .try_into()
                .map_err(|e| format!("Failed to parse inner definition for {k}: {e}"))?;

            out.insert(k.as_str(), def);
        }
    }
    Ok(out)
}

fn generate_comment(
    description: Option<&str>,
    enum_values: Option<&[&str]>,
    required: bool,
) -> String {
    let enum_comment =
        enum_values.map(|values| format!("should be one of: [{}]", values.join(", ")));
    let optional = if required { None } else { Some("(optional)") };

    match (description, enum_comment, optional) {
        (Some(desc), Some(enum_desc), Some(opt)) => format!("// {desc}, {enum_desc} {opt}"),
        (Some(desc), Some(enum_desc), None) => format!("// {desc}, {enum_desc}"),
        (Some(desc), None, Some(opt)) => format!("// {desc} {opt}"),
        (None, Some(enum_desc), Some(opt)) => format!("// {enum_desc} {opt}"),
        (Some(desc), None, None) => format!("// {desc}"),
        (None, Some(enum_desc), None) => format!("// {enum_desc}"),
        (None, None, Some(opt)) => format!("// {opt}"),
        (None, None, None) => String::new(),
    }
}

fn describe_schema(
    schema: &Schema,
    required: bool,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if depth > 10 {
        return Ok("object // Too deep".into());
    }

    if let Some(b) = schema.as_bool() {
        return Ok(if b { "any".into() } else { "never".into() });
    }

    let Some(obj) = schema.as_object() else {
        return Err("Schema is not an object or a boolean".into());
    };

    if let Some(Value::String(reference)) = obj.get("$ref") {
        return resolve_reference(reference, required, definitions, depth);
    }

    let instance_type = match obj.get("type") {
        Some(Value::String(t)) => Some(t.as_str()),
        Some(Value::Array(arr)) => {
            let first = arr.iter().filter_map(|v| v.as_str()).next();
            if first.is_none() {
                log::warn!("Type array is empty or contains non-string values");
            }
            first
        }
        _ => None,
    }
    .ok_or_else(|| String::from("Field type is missing"))?;

    let description = obj.get("description").and_then(|v| v.as_str());
    let enum_values = obj
        .get("enum")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>());
    let comment = generate_comment(description, enum_values.as_deref(), required);

    let full_description = match instance_type {
        "null" => "{} // An empty object".to_string(),
        "boolean" => format!("bool {comment}"),
        "number" => format!("number {comment}"),
        "integer" => format!("integer {comment}"),
        "string" => format!("string {comment}"),
        "object" => describe_object(obj, &comment, definitions, depth)?,
        "array" => describe_array(obj, &comment, definitions, depth)?,
        other => return Err(format!("Unsupported type: {other}")),
    };
    Ok(full_description)
}

fn describe_object(
    obj: &serde_json::Map<String, Value>,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if obj.get("patternProperties").is_some() {
        log::warn!("Pattern properties are not supported, they will be ignored");
    }

    let required: HashSet<&str> = obj
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let Some(properties) = obj.get("properties").and_then(|v| v.as_object()) else {
        return Ok(format!("object {comment} {{}}"));
    };

    let properties = properties
        .iter()
        .map(|(name, schema)| {
            let is_required = required.contains(name.as_str());
            let subschema = schema
                .try_into()
                .map_err(|e| format!("Invalid schema for property {name}: {e}"))?;
            let description = describe_schema(subschema, is_required, definitions, depth + 1)?;
            Ok(format!("{name}: {description}"))
        })
        .collect::<Result<Vec<_>, String>>()?
        .join("\n");

    Ok(formatdoc! {"
        object {comment}
        {{
        {}
        }}",
        add_indent(&properties, 4, true)
    })
}

fn describe_array(
    obj: &serde_json::Map<String, Value>,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    let Some(items) = obj.get("items") else {
        return Ok(format!("[] {comment}"));
    };

    let items_schema = match items {
        Value::Array(arr) => {
            log::warn!("Union types for array items are not supported, using the first one");
            if let Some(first) = arr.first() {
                first
            } else {
                return Ok(format!("[] {comment}"));
            }
        }
        other => other,
    };
    let items_schema = items_schema
        .try_into()
        .map_err(|e| format!("Invalid schema for array items: {e}"))?;

    let item_description = format!(
        "items: {}",
        describe_schema(items_schema, true, definitions, depth + 1)?
    );

    Ok(formatdoc! {"
        array {comment}
        [
        {}
        ]",
        add_indent(&item_description, 4, true)
    })
}

fn resolve_reference(
    reference: &str,
    required: bool,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if reference == "#" {
        return Ok("object // Same as the root object".into());
    }

    let re = Regex::new(r"^#\/(?:\$defs|definitions)\/(.+)$").unwrap();

    let Some(captures) = re.captures(reference) else {
        return Err(format!("Invalid reference {reference}"));
    };

    let definition_name = captures.get(1).ok_or("Invalid reference")?.as_str();

    let Some(definition) = definitions.get(definition_name) else {
        return Err(format!("Definition {definition_name} not found"));
    };

    describe_schema(definition, required, definitions, depth)
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use indoc::indoc;
    use schemars::{schema_for, JsonSchema};

    use crate::tools::DefaultFunctionInput;

    use super::*;

    #[test]
    fn test_describe_parameters() {
        let schema = schema_for!(DefaultFunctionInput);
        let description = describe_parameters(&schema).unwrap();

        assert_eq!(description, "string // The input for the tool");
    }

    #[test]
    fn test_describe_parameters_for_object() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The input for the tool")]
        pub struct TestObject {
            #[schemars(description = "The name of the person")]
            pub name: String,
            #[schemars(description = "The age of the person")]
            pub age: u32,
        }

        let schema = schema_for!(TestObject);
        let description = describe_parameters(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                object // The input for the tool
                {
                    name: string // The name of the person
                    age: integer // The age of the person
                }"}
        );
    }

    #[test]
    fn test_describe_parameters_for_array() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The list of numbers")]
        pub struct TestArray(pub Vec<u32>);

        let schema = schema_for!(TestArray);
        let description = describe_parameters(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                array // The list of numbers
                [
                    items: integer 
                ]"}
        );
    }

    #[test]
    fn test_describe_parameters_complex() {
        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "The input for the tool")]
        pub struct ComplexInput {
            #[schemars(description = "The name of the person")]
            pub name: String,
            #[schemars(description = "The age of the person")]
            pub age: u32,
            #[schemars(description = "The list of phone numbers")]
            pub numbers: Vec<PhoneNumber>,
        }

        #[derive(JsonSchema)]
        #[serde(deny_unknown_fields)]
        #[schemars(description = "A phone number")]
        pub struct PhoneNumber {
            #[schemars(description = "The phone number")]
            pub number: String,
            #[schemars(description = "The type of the phone number")]
            #[serde(rename = "type")]
            pub type_: String,
        }

        let schema = schema_for!(ComplexInput);
        let description = describe_parameters(&schema).unwrap();

        assert_eq!(
            description,
            indoc! {"
                object // The input for the tool
                {
                    name: string // The name of the person
                    age: integer // The age of the person
                    numbers: array // The list of phone numbers
                    [
                        items: object // A phone number
                        {
                            number: string // The phone number
                            type: string // The type of the phone number
                        }
                    ]
                }"}
        );
    }
}
