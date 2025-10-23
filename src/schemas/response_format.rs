use async_openai::types::ResponseFormatJsonSchema;
use async_openai::types::responses::TextResponseFormat;

/// A parameter to control the format that the model must output.
///
/// Corresponds to [`ResponseFormat`](async_openai::types::ResponseFormat) for the chat completions
/// API and [`TextResponseFormat`](async_openai::types::responses::TextResponseFormat) for the
/// responses API.
#[derive(Clone, Debug)]
pub struct ResponseFormat(TextResponseFormat);

impl ResponseFormat {
    /// The model will output plain text.
    pub const TEXT: Self = ResponseFormat(TextResponseFormat::Text);

    /// The model will output a valid JSON object.
    pub const JSON_OBJECT: Self = ResponseFormat(TextResponseFormat::JsonObject);

    /// The model will output plain text.
    #[must_use]
    #[inline]
    pub const fn text() -> Self {
        Self::TEXT
    }

    /// The model will output a valid JSON object.
    #[must_use]
    #[inline]
    pub const fn json_object() -> Self {
        Self::JSON_OBJECT
    }

    /// The model will output a JSON object that conforms to the provided JSON schema.
    ///
    /// This function does not validate whether the provided schema is a valid JSON schema. The
    /// OpenAI API may reject the request if the schema is invalid.
    #[must_use]
    pub fn json_schema(
        name: impl Into<String>,
        description: Option<String>,
        schema: serde_json::Value,
        strict: bool,
    ) -> Self {
        ResponseFormat(TextResponseFormat::JsonSchema(ResponseFormatJsonSchema {
            name: name.into(),
            description,
            schema: Some(schema),
            strict: Some(strict),
        }))
    }
}

impl From<ResponseFormat> for async_openai::types::ResponseFormat {
    fn from(value: ResponseFormat) -> Self {
        match value.0 {
            TextResponseFormat::Text => async_openai::types::ResponseFormat::Text,
            TextResponseFormat::JsonObject => async_openai::types::ResponseFormat::JsonObject,
            TextResponseFormat::JsonSchema(s) => async_openai::types::ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    name: s.name,
                    description: s.description,
                    schema: s.schema,
                    strict: s.strict,
                },
            },
        }
    }
}

impl From<async_openai::types::ResponseFormat> for ResponseFormat {
    fn from(value: async_openai::types::ResponseFormat) -> Self {
        match value {
            async_openai::types::ResponseFormat::Text => ResponseFormat(TextResponseFormat::Text),
            async_openai::types::ResponseFormat::JsonObject => {
                ResponseFormat(TextResponseFormat::JsonObject)
            }
            async_openai::types::ResponseFormat::JsonSchema { json_schema } => {
                ResponseFormat(TextResponseFormat::JsonSchema(ResponseFormatJsonSchema {
                    name: json_schema.name,
                    description: json_schema.description,
                    schema: json_schema.schema,
                    strict: json_schema.strict,
                }))
            }
        }
    }
}

impl From<ResponseFormat> for TextResponseFormat {
    fn from(value: ResponseFormat) -> Self {
        value.0
    }
}

impl From<TextResponseFormat> for ResponseFormat {
    fn from(value: TextResponseFormat) -> Self {
        ResponseFormat(value)
    }
}
