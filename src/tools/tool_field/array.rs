use serde_json::{Map, Value};

use crate::utils::helper::add_indent;

use super::{BooleanField, IntegerField, NumberField, StringField, ToolField};

pub struct ArrayField {
    name: String,
    description: Option<String>,
    required: bool,
    field: Box<dyn ToolField>,
}

impl Clone for ArrayField {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            description: self.description.clone(),
            required: self.required,
            field: self.field.clone_box(),
        }
    }
}

impl ArrayField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
        field: Box<dyn ToolField>,
    ) -> Self {
        ArrayField {
            name: name.into(),
            description: description.map(Into::into),
            required,
            field,
        }
    }

    pub fn new_string_array(name: impl Into<String>) -> Self {
        ArrayField::new_full(name, None::<&str>, true, StringField::new("items").into())
    }

    pub fn new_string_enum_array(
        name: impl Into<String>,
        r#enum: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        ArrayField::new_full(
            name,
            None::<&str>,
            true,
            StringField::new("items").r#enum(r#enum).into(),
        )
    }

    pub fn new_integer_array(name: impl Into<String>) -> Self {
        ArrayField::new_full(name, None::<&str>, true, IntegerField::new("items").into())
    }

    pub fn new_integer_enum_array(
        name: impl Into<String>,
        r#enum: impl IntoIterator<Item = i64>,
    ) -> Self {
        ArrayField::new_full(
            name,
            None::<&str>,
            true,
            IntegerField::new("items").r#enum(r#enum).into(),
        )
    }

    pub fn new_number_array(name: impl Into<String>) -> Self {
        ArrayField::new_full(name, None::<&str>, true, NumberField::new("items").into())
    }

    pub fn new_number_enum_array(
        name: impl Into<String>,
        r#enum: impl IntoIterator<Item = f64>,
    ) -> Self {
        ArrayField::new_full(
            name,
            None::<&str>,
            true,
            NumberField::new("items").r#enum(r#enum).into(),
        )
    }

    pub fn new_boolean_array(name: impl Into<String>) -> Self {
        ArrayField::new_full(name, None::<&str>, true, BooleanField::new("items").into())
    }

    pub fn new_boolean_enum_array(
        name: impl Into<String>,
        r#enum: impl IntoIterator<Item = bool>,
    ) -> Self {
        ArrayField::new_full(
            name,
            None::<&str>,
            true,
            BooleanField::new("items").r#enum(r#enum).into(),
        )
    }

    pub fn new_items_array(name: impl Into<String>, field: Box<dyn ToolField>) -> Self {
        ArrayField::new_full(name, None::<&str>, true, field)
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
}

impl ToolField for ArrayField {
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

        fields.insert("type".into(), "array".into());
        fields.insert("items".into(), self.field.to_openai_field());
        if let Some(description) = self.description() {
            fields.insert("description".into(), description.into());
        }

        Value::Object(fields)
    }

    fn to_plain_description(&self) -> String {
        let type_info = if self.required {
            "array"
        } else {
            "array, optional"
        };

        let items_description = add_indent(&self.field.to_plain_description(), 4, true);

        match &self.description {
            Some(description) => format!(
                "{} ({}): {}\n{}",
                self.name, type_info, description, items_description
            ),
            None => format!("{} ({})\n{}", self.name, type_info, items_description),
        }
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<ArrayField> for Box<dyn ToolField> {
    fn from(value: ArrayField) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_array_field_plain_description() {
        let field = ArrayField::new_integer_array("test").description("test description");

        assert_eq!(
            field.to_plain_description(),
            "test (array): test description\n    items (integer)"
        );

        let field_optional = ArrayField::new_string_array("test")
            .description("test description")
            .optional();
        assert_eq!(
            field_optional.to_plain_description(),
            "test (array, optional): test description\n    items (string)"
        );

        let field_optional_no_description =
            ArrayField::new_number_enum_array("test", [1.0f64, 2f64]).optional();
        assert_eq!(
            field_optional_no_description.to_plain_description(),
            "test (array, optional)\n    items (number): should be one of [1, 2]"
        );
    }

    #[test]
    fn test_array_field_openai() {
        let field = ArrayField::new_integer_array("test").description("test description");
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "array",
                "description": "test description",
                "items": {
                    "type": "integer"
                }
            })
        );

        let field_optional = ArrayField::new_string_array("test")
            .description("test description")
            .optional();
        assert_eq!(
            field_optional.to_openai_field(),
            json!({
                "type": "array",
                "description": "test description",
                "items": {
                    "type": "string"
                }
            })
        );

        let field_optional_no_description = ArrayField::new_number_array("test").optional();
        assert_eq!(
            field_optional_no_description.to_openai_field(),
            json!({
                "type": "array",
                "items": {
                    "type": "number"
                }
            })
        );
    }
}
