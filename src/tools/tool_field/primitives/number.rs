use crate::tools::tool_field::ToolField;

use super::ToolFieldPrimitive;

#[derive(Clone)]
pub struct NumberField {
    name: String,
    description: Option<String>,
    required: bool,
    r#enum: Option<Vec<f64>>,
}

impl NumberField {
    pub fn new_full(
        name: impl Into<String>,
        description: Option<impl Into<String>>,
        required: bool,
        r#enum: Option<Vec<f64>>,
    ) -> Self {
        NumberField {
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

    pub fn r#enum(self, r#enum: impl IntoIterator<Item = f64>) -> Self {
        Self {
            r#enum: Some(r#enum.into_iter().collect()),
            ..self
        }
    }
}

impl ToolFieldPrimitive for NumberField {
    type FieldType = f64;

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
        "number"
    }

    fn r#enum(&self) -> Option<&Vec<f64>> {
        self.r#enum.as_ref()
    }

    fn clone_box(&self) -> Box<dyn ToolField> {
        Box::new(self.clone())
    }
}

impl From<NumberField> for Box<dyn ToolField> {
    fn from(value: NumberField) -> Self {
        Box::new(value)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::tools::tool_field::ToolField;

    #[test]
    fn test_number_field_plain_description() {
        let field = NumberField::new("test").description("test description");
        assert_eq!(
            field.to_plain_description(),
            "test (number): test description"
        );

        let optional_field = NumberField::new("test")
            .description("test description")
            .optional();
        assert_eq!(
            optional_field.to_plain_description(),
            "test (number, optional): test description"
        );

        let enum_field = NumberField::new("test")
            .description("test description")
            .r#enum([0.1, 3f64]);
        assert_eq!(
            enum_field.to_plain_description(),
            "test (number): test description, should be one of [0.1, 3]"
        );

        let enum_field_without_description = NumberField::new("test").r#enum([3.2, 5f64]);
        assert_eq!(
            enum_field_without_description.to_plain_description(),
            "test (number): should be one of [3.2, 5]"
        );

        let field_without_description = NumberField::new("test");
        assert_eq!(
            field_without_description.to_plain_description(),
            "test (number)"
        )
    }

    #[test]
    fn test_boolean_field_openai() {
        let field = NumberField::new("test").description("test description");
        assert_eq!(
            field.to_openai_field(),
            json!({
                "type": "number",
                "description": "test description"
            })
        );

        let enum_field = NumberField::new("test")
            .description("test description")
            .r#enum([3.1, 3.12]);
        assert_eq!(
            enum_field.to_openai_field(),
            json!({
                "type": "number",
                "description": "test description",
                "enum": [3.1, 3.12]
            })
        );

        let field_without_description = NumberField::new("test");
        assert_eq!(
            field_without_description.to_openai_field(),
            json!({
                "type": "number"
            })
        );
    }
}
