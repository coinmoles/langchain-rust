use std::string::String;

use async_trait::async_trait;
use schemars::{gen::SchemaSettings, schema::RootSchema, schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::tools::ToolOutput;

#[async_trait]
pub trait Tool: Send + Sync {
    /// The input type for the tool.
    ///
    /// `JsonSchema` must be implemented for the type,
    /// which can be done using the derive macro from the `schemars` crate.
    ///
    /// If the type has fields of non-primitive types,
    /// you should implement `inline_subschema` and return `true`.
    type Input: JsonSchema + DeserializeOwned + Send + Sync;

    /// The output type for the tool. Recommended to use `String` or `Vec<String>` for simple tools.
    type Output: Into<ToolOutput> + Send + Sync;

    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// Returns whether the tool has an subschema that should be inlined in the schema.
    ///
    /// If not implemented, it will default to `false`.
    ///
    /// Implement this method and return `true` if the input type has fields of non-primitive types that should be inlined in the schema.
    fn inline_subschema(&self) -> bool {
        false
    }

    /// JSON schema for the tool input parameters.
    ///
    /// Used for OpenAI function call.
    ///
    /// You don't need to implement this method as it is automatically generated based on the `Input` type.
    fn parameters(&self) -> RootSchema {
        if self.inline_subschema() {
            SchemaSettings::default()
                .with(|s| s.inline_subschemas = true)
                .into_generator()
                .into_root_schema_for::<Self::Input>()
        } else {
            schema_for!(Self::Input)
        }
    }

    /// Whether the tool should be strict in its input validation.
    ///
    /// Used for OpenAI function call.
    ///
    /// If not implemented, it will default to `false`.
    fn strict(&self) -> bool {
        false
    }

    /// Executes the core functionality of the tool.
    ///
    /// Example implementation:
    /// ```rust
    /// // type Input = (usize, usize);
    /// // type Output = String;
    /// async fn run(
    ///     &self,
    ///     input: (usize, usize)
    /// ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    ///     let result = self.0 + self.1;
    ///     Ok(result.to_string())
    /// }
    /// ```
    async fn run(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>>;

    /// Parses the input string, which could be a JSON value or a raw string, depending on the LLM model.
    ///
    /// If not implemented, it will default to parsing the input as a JSON value.
    async fn parse_input(&self, input: Value) -> Result<Self::Input, serde_json::Error> {
        serde_json::from_value(input)
    }

    /// The usage limit for the tool.
    ///
    /// If not implemented, it will default to `None`, meaning no limit.
    fn usage_limit(&self) -> Option<usize> {
        None
    }
}
