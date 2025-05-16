use std::collections::BTreeMap;

use indoc::formatdoc;
use regex::Regex;
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, RootSchema, Schema, SchemaObject, SingleOrVec,
};

use crate::utils::helper::add_indent;

pub fn describe_parameters(parameters: &RootSchema) -> Result<String, String> {
    let definitions = parameters
        .definitions
        .iter()
        .map(|(k, v)| (k.as_str(), v))
        .collect();

    describe_schema_object(&parameters.schema, true, &definitions, 0)
}

fn generate_comment(
    description: Option<&str>,
    enum_values: Option<&Vec<serde_json::Value>>,
    required: bool,
) -> String {
    let enum_comment = enum_values.map(|values| {
        format!(
            "should be one of: [{}]",
            values
                .iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
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
    match schema {
        Schema::Object(schema_object) => {
            describe_schema_object(schema_object, required, definitions, depth)
        }
        Schema::Bool(true) => Ok("any".into()),
        Schema::Bool(false) => Ok("never".into()),
    }
}

fn describe_schema_object(
    schema_object: &SchemaObject,
    required: bool,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if depth > 10 {
        return Ok("object // Too deep".into());
    }

    if let Some(ref reference) = &schema_object.reference {
        return resolve_reference(reference, required, definitions, depth);
    }

    let Some(ref instance_type) = schema_object.instance_type else {
        return Err("Field type is missing".into());
    };

    let instance_type = match instance_type {
        SingleOrVec::Single(instance_type) => instance_type,
        SingleOrVec::Vec(instance_types) => {
            log::warn!("Union types are not supported, using the first one");
            instance_types.first().ok_or("Field type is empty")?
        }
    };

    let description = schema_object
        .metadata
        .as_ref()
        .and_then(|m| m.description.as_deref());
    let enum_values = schema_object.enum_values.as_ref();
    let comment = generate_comment(description, enum_values, required);

    let full_description = match instance_type {
        InstanceType::Null => "{} // An empty object".to_string(),
        InstanceType::Boolean => format!("bool {comment}"),
        InstanceType::Number => format!("number {comment}"),
        InstanceType::Integer => format!("integer {comment}"),
        InstanceType::String => format!("string {comment}"),
        InstanceType::Object => describe_object(
            schema_object.object.as_ref().ok_or("Not an object")?,
            &comment,
            definitions,
            depth,
        )?,
        InstanceType::Array => describe_array(
            schema_object.array.as_ref().ok_or("Not an array")?,
            &comment,
            definitions,
            depth,
        )?,
    };

    Ok(full_description)
}

fn describe_object(
    object: &ObjectValidation,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    if !object.pattern_properties.is_empty() {
        log::warn!("Pattern properties are not supported, they will be ignored");
    }

    let properties = object
        .properties
        .iter()
        .map(|(k, v)| -> Result<String, String> {
            let required = object.required.contains(k);
            let description = describe_schema(v, required, definitions, depth + 1)?;
            Ok(format!("{k}: {description}"))
        })
        .collect::<Result<Vec<_>, _>>()?
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
    array: &ArrayValidation,
    comment: &str,
    definitions: &BTreeMap<&str, &Schema>,
    depth: usize,
) -> Result<String, String> {
    let Some(ref items) = array.items else {
        return Ok(format!("[] {comment}"));
    };

    let items_schema = match items {
        SingleOrVec::Single(items) => items,
        SingleOrVec::Vec(items) => {
            log::warn!("Union types for array items are not supported, using the first one");
            if let Some(first) = items.first() {
                first
            } else {
                return Ok(format!("[] {comment}"));
            }
        }
    };

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

    let re = Regex::new(r"^#\/definitions\/(.+)$").unwrap();

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
mod tests {
    use indoc::indoc;
    use schemars::{schema_for, JsonSchema};

    use crate::tools::input::DefaultToolInput;

    use super::*;

    #[test]
    fn test_describe_parameters() {
        let schema = schema_for!(DefaultToolInput);
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
