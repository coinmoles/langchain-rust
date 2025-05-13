use std::cmp::Ordering;

use serde_json::{Map, Value};

use crate::utils::helper::add_indent;

use super::ToolField;

pub struct ObjectField {
    name: String,
    description: Option<String>,
    required: bool,
    properties: Vec<Box<dyn ToolField>>,
    additional_properties: Option<bool>,
}

impl Clone for ObjectField {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            required: self.required,
            properties: self.properties.iter().map(|p| p.clone_box()).collect(),
            additional_properties: self.additional_properties,
        }
    }
}

impl ObjectField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<String>,
        required: bool,
        mut properties: Vec<Box<dyn ToolField>>,
        additional_properties: Option<bool>,
    ) -> Self {
        properties.sort_by(|a, b| match (a.required(), b.required()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => Ordering::Equal,
        });

        Self {
            name: name.into(),
            description,
            required,
            properties,
            additional_properties,
        }
    }

    pub fn new(
        name: impl Into<String>,
        properties: impl IntoIterator<Item = Box<dyn ToolField>>,
    ) -> Self {
        Self::new_full(name, None, true, properties.into_iter().collect(), None)
    }

    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    pub fn required(self) -> Self {
        Self {
            required: true,
            ..self
        }
    }

    pub fn optional(self) -> Self {
        Self {
            required: false,
            ..self
        }
    }

    pub fn additional_properties(self, additional_properties: bool) -> Self {
        Self {
            additional_properties: Some(additional_properties),
            ..self
        }
    }

    pub fn properties_description(&self) -> String {
        let properties = self
            .properties
            .iter()
            .map(|property| property.to_plain_description())
            .collect::<Vec<_>>()
            .join(",\n");

        let properties = add_indent(&properties, 4, true);

        if properties.is_empty() {
            "{}".into()
        } else {
            format!("{{\n{}\n}}", properties)
        }
    }
}

impl ToolField for ObjectField {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn required(&self) -> bool {
        self.required
    }

    fn to_openai_field(&self) -> Value {
        let mut fields = Map::<String, Value>::new();

        fields.insert("type".into(), "object".into());
        fields.insert(
            "properties".into(),
            Map::from_iter(
                self.properties
                    .iter()
                    .map(|property| (property.name().into(), property.to_openai_field())),
            )
            .into(),
        );
        fields.insert(
            "required".into(),
            self.properties
                .iter()
                .filter(|property| property.required())
                .map(|property| property.name())
                .collect::<Vec<_>>()
                .into(),
        );
        if let Some(description) = self.description() {
            fields.insert("description".into(), description.into());
        }

        let additional_properties = self.additional_properties.unwrap_or(true);
        fields.insert("additionalProperties".into(), additional_properties.into());

        Value::Object(fields)
    }

    fn to_plain_description(&self) -> String {
        let type_info = if self.required {
            "object"
        } else {
            "object, optional"
        };

        format!(
            "{} ({}): {}",
            self.name,
            type_info,
            self.properties_description()
        )
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<ObjectField> for Box<dyn ToolField> {
    fn from(value: ObjectField) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::tools::tool_field::{IntegerField, StringField};

    use super::*;
    use indoc::indoc;
    use serde_json::json;

    #[test]
    fn test_object_field_properties_description() {
        let field = ObjectField::new("test", []);
        assert_eq!(field.properties_description(), "{}");

        let field_complicated = ObjectField::new(
            "test",
            [
                StringField::new("query")
                    .description("A query to search for")
                    .into(),
                IntegerField::new("limit")
                    .description("Max number of articles to search")
                    .optional()
                    .into(),
            ],
        )
        .optional();
        assert_eq!(
            field_complicated.properties_description(),
            indoc! {"
            {
                query (string): A query to search for,
                limit (integer, optional): Max number of articles to search
            }"}
        )
    }

    #[test]
    fn test_object_field_plain_description() {
        let field = ObjectField::new("test", []);
        assert_eq!(field.to_plain_description(), "test (object): {}");

        let field_complicated = ObjectField::new(
            "test",
            [
                StringField::new("query")
                    .description("A query to search for")
                    .into(),
                IntegerField::new("limit")
                    .description("Max number of articles to search")
                    .optional()
                    .into(),
            ],
        )
        .optional();
        assert_eq!(
            field_complicated.to_plain_description(),
            indoc! {"
            test (object, optional): {
                query (string): A query to search for,
                limit (integer, optional): Max number of articles to search
            }"}
        )
    }

    #[test]
    fn test_object_field_openai() {
        let field = ObjectField::new("test", []);
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "object",
                "properties": {},
                "required": [],
                "additionalProperties": true
            })
        );

        let field_complicated = ObjectField::new(
            "test",
            [
                StringField::new("query")
                    .description("A query to search for")
                    .into(),
                IntegerField::new("limit")
                    .description("Max number of articles to search")
                    .optional()
                    .into(),
            ],
        )
        .optional();
        assert_eq!(
            field_complicated.to_openai_field(),
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "A query to search for"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max number of articles to search"
                    }
                },
                "required": ["query"],
                "additionalProperties": true
            })
        )
    }
}
