use std::fmt::Display;
use std::string::String;

use async_trait::async_trait;
use schemars::{gen::SchemaSettings, schema::RootSchema, schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use serde_json::Value;

#[async_trait]
pub trait ToolFunction: Send + Sync {
    type Input: JsonSchema + DeserializeOwned + Send + Sync;
    type Output: Display + Send + Sync;

    fn name(&self) -> String;

    fn description(&self) -> String;

    fn inline_subschema(&self) -> bool {
        false
    }

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

    fn strict(&self) -> bool {
        false
    }

    /// Executes the core functionality of the tool.
    ///
    /// Example implementation:
    /// ```rust,ignore
    /// async fn run(&self, input: ToolInput) -> Result<String, Box<dyn Error>> {
    ///     self.simple_search(input).await
    /// }
    /// ```
    async fn run(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>>;

    /// Parses the input string, which could be a JSON value or a raw string, depending on the LLM model.
    ///
    /// Implement this function to extract the parameters needed for your tool. If a simple
    /// string is sufficient, the default implementation can be used.
    async fn parse_input(&self, input: Value) -> Result<Self::Input, serde_json::Error> {
        serde_json::from_value(input)
    }

    fn usage_limit(&self) -> Option<usize> {
        None
    }
}
