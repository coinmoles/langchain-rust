use crate::tools::tool_field::ToolField;

use super::ToolFieldPrimitive;

#[derive(Clone)]
pub struct BooleanField {
    name: String,
    description: Option<String>,
    required: bool,
    r#enum: Option<Vec<bool>>,
}

impl BooleanField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
        r#enum: Option<Vec<bool>>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.map(Into::into),
            required,
            r#enum,
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

    pub fn r#enum(self, r#enum: impl IntoIterator<Item = bool>) -> Self {
        Self {
            r#enum: Some(r#enum.into_iter().collect()),
            ..self
        }
    }
}

impl ToolFieldPrimitive for BooleanField {
    type FieldType = bool;

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
        "boolean"
    }

    fn r#enum(&self) -> Option<&Vec<bool>> {
        self.r#enum.as_ref()
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<BooleanField> for Box<dyn ToolField> {
    fn from(value: BooleanField) -> Self {
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
        let field = BooleanField::new("test").description("test description");
        assert_eq!(
            field.to_plain_description(),
            "test (boolean): test description"
        );

        let optional_field = BooleanField::new("test")
            .description("test description")
            .optional();
        assert_eq!(
            optional_field.to_plain_description(),
            "test (boolean, optional): test description"
        );

        let enum_field = BooleanField::new("test")
            .description("test description")
            .r#enum([true, false]);
        assert_eq!(
            enum_field.to_plain_description(),
            "test (boolean): test description, should be one of [true, false]"
        );

        let enum_field_without_description = BooleanField::new("test").r#enum([true, false]);
        assert_eq!(
            enum_field_without_description.to_plain_description(),
            "test (boolean): should be one of [true, false]"
        );

        let field_without_description = BooleanField::new("test");
        assert_eq!(
            field_without_description.to_plain_description(),
            "test (boolean)"
        )
    }

    #[test]
    fn test_boolean_field_openai() {
        let field = BooleanField::new("test").description("test description");
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "boolean",
                "description": "test description"
            })
        );

        let enum_field = BooleanField::new("test")
            .description("test description")
            .r#enum([true, false]);
        assert_eq!(
            enum_field.to_openai_field(),
            json!({
                "type": "boolean",
                "description": "test description",
                "enum": [true, false]
            })
        );

        let field_without_description = BooleanField::new("test");
        assert_eq!(
            field_without_description.to_openai_field(),
            json!({
                "type": "boolean"
            })
        );
    }
}
