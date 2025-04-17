use crate::tools::tool_field::ToolField;

use super::ToolFieldPrimitive;

#[derive(Clone)]
pub struct IntegerField {
    name: String,
    description: Option<String>,
    required: bool,
    r#enum: Option<Vec<i64>>,
}

impl IntegerField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
        r#enum: Option<Vec<i64>>,
    ) -> Self {
        IntegerField {
            name: name.into(),
            description: description.map(Into::into),
            required,
            r#enum: r#enum.map(|options| {
                let mut options = options.clone();
                options.dedup();
                options
            }),
        }
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self::new_full(name, None::<&str>, true, None)
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

    pub fn r#enum(self, r#enum: impl IntoIterator<Item = i64>) -> Self {
        Self {
            r#enum: Some(r#enum.into_iter().collect()),
            ..self
        }
    }
}

impl ToolFieldPrimitive for IntegerField {
    type FieldType = i64;

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
        "integer"
    }

    fn r#enum(&self) -> Option<&Vec<i64>> {
        self.r#enum.as_ref()
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<IntegerField> for Box<dyn ToolField> {
    fn from(value: IntegerField) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::tools::tool_field::ToolField;

    #[test]
    fn test_integer_field_plain_description() {
        let field = IntegerField::new("test").description("test description");
        assert_eq!(
            field.to_plain_description(),
            "test (integer): test description"
        );

        let optional_field = IntegerField::new("test")
            .description("test description")
            .optional();
        assert_eq!(
            optional_field.to_plain_description(),
            "test (integer, optional): test description"
        );

        let enum_field = IntegerField::new("test")
            .description("test description")
            .r#enum([0, 1, 3]);
        assert_eq!(
            enum_field.to_plain_description(),
            "test (integer): test description, should be one of [0, 1, 3]"
        );

        let enum_field_without_description = IntegerField::new("test").r#enum([0, 1, 5]);
        assert_eq!(
            enum_field_without_description.to_plain_description(),
            "test (integer): should be one of [0, 1, 5]"
        );

        let field_without_description = IntegerField::new("test");
        assert_eq!(
            field_without_description.to_plain_description(),
            "test (integer)"
        )
    }

    #[test]
    fn test_integer_field_openai() {
        let field = IntegerField::new("test").description("test description");
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "integer",
                "description": "test description"
            })
        );

        let enum_field = IntegerField::new("test")
            .description("test description")
            .r#enum([4, 8]);
        assert_eq!(
            enum_field.to_openai_field(),
            json!({
                "type": "integer",
                "description": "test description",
                "enum": [4, 8]
            })
        );

        let field_without_description = IntegerField::new("test");
        assert_eq!(
            field_without_description.to_openai_field(),
            json!({
                "type": "integer"
            })
        );
    }
}
