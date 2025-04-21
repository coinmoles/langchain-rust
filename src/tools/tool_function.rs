use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{
    tool_field::{StringField, ToolParameters},
    Tool,
};

#[async_trait]
pub trait ToolFunction: Send + Sync {
    type Input: Send + Sync + DeserializeOwned;
    type Result: Display + Send + Sync;

    fn name(&self) -> String;

    fn description(&self) -> String;

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new([StringField::new("input")
            .description("The input for the tool")
            .into()])
        .additional_properties(false)
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
    async fn run(&self, input: Self::Input) -> Result<Self::Result, Box<dyn Error + Send + Sync>>;

    /// Parses the input string, which could be a JSON value or a raw string, depending on the LLM model.
    ///
    /// Implement this function to extract the parameters needed for your tool. If a simple
    /// string is sufficient, the default implementation can be used.
    async fn parse_input(&self, input: Value) -> Result<Self::Input, Box<dyn Error + Send + Sync>> {
        let result = serde_json::from_value(input)?;

        Ok(result)
    }

    fn usage_limit(&self) -> Option<usize> {
        None
    }
}

#[async_trait]
impl<T> Tool for T
where
    T: ToolFunction,
{
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description()
    }

    fn parameters(&self) -> ToolParameters {
        self.parameters()
    }

    fn strict(&self) -> bool {
        self.strict()
    }

    async fn call(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let input = self.parse_input(input).await?;
        let result = self.run(input).await?;

        Ok(result.to_string())
    }

    fn usage_limit(&self) -> Option<usize> {
        self.usage_limit()
    }
}
