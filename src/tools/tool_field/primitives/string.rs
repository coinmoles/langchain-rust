use crate::tools::tool_field::ToolField;

use super::ToolFieldPrimitive;

#[derive(Clone)]
pub struct StringField {
    name: String,
    description: Option<String>,
    required: bool,
    r#enum: Option<Vec<String>>,
}

impl StringField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
        r#enum: Option<impl IntoIterator<Item = impl Into<String>>>,
    ) -> Self {
        StringField {
            name: name.into(),
            description: description.map(Into::into),
            required,
            r#enum: r#enum.map(|options| {
                let mut options = options.into_iter().map(Into::into).collect::<Vec<_>>();
                options.dedup();
                options
            }),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self::new_full(name, None::<&str>, true, None::<Vec<&str>>)
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

    pub fn r#enum(self, r#enum: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            r#enum: Some(r#enum.into_iter().map(Into::into).collect()),
            ..self
        }
    }
}

impl ToolFieldPrimitive for StringField {
    type FieldType = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn required(&self) -> bool {
        self.required
    }

    fn type_name(&self) -> &str {
        "string"
    }

    fn r#enum(&self) -> Option<&Vec<String>> {
        self.r#enum.as_ref()
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<StringField> for Box<dyn ToolField> {
    fn from(value: StringField) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::tools::tool_field::ToolField;

    #[test]
    fn test_boolean_field_plain_description() {
        let field = StringField::new("test").description("test description");
        assert_eq!(
            field.to_plain_description(),
            "test (string): test description"
        );

        let optional_field = StringField::new("test")
            .description("test description")
            .optional();
        assert_eq!(
            optional_field.to_plain_description(),
            "test (string, optional): test description"
        );

        let enum_field = StringField::new("test")
            .description("test description")
            .r#enum(["lala", "blah"]);
        assert_eq!(
            enum_field.to_plain_description(),
            "test (string): test description, should be one of [lala, blah]"
        );

        let enum_field_without_description = StringField::new("test").r#enum(["true", "blah"]);
        assert_eq!(
            enum_field_without_description.to_plain_description(),
            "test (string): should be one of [true, blah]"
        );

        let field_without_description = StringField::new("test");
        assert_eq!(
            field_without_description.to_plain_description(),
            "test (string)"
        )
    }

    #[test]
    fn test_boolean_field_openai() {
        let field = StringField::new("test").description("test description");
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "string",
                "description": "test description"
            })
        );

        let enum_field = StringField::new("test")
            .description("test description")
            .r#enum(["lala", "blah"]);
        assert_eq!(
            enum_field.to_openai_field(),
            json!({
                "type": "string",
                "description": "test description",
                "enum": ["lala", "blah"]
            })
        );

        let field_without_description = StringField::new("test");
        assert_eq!(
            field_without_description.to_openai_field(),
            json!({
                "type": "string"
            })
        );
    }
}
