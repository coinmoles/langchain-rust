use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use async_trait::async_trait;
use derive_new::new;
use serde_json::Value;

use super::{
    tool_field::{ObjectField, StringField},
    Tool,
};

#[async_trait]
pub trait ToolFunction: Default + Send + Sync + Into<Arc<dyn Tool>> {
    type Input: Send + Sync;
    type Result: Display + Send + Sync;

    fn name(&self) -> String;

    fn description(&self) -> String;

    fn parameters(&self) -> ObjectField {
        ObjectField::new_tool_input(
            vec![
                StringField::new("input", Some("The input for the tool".into()), true, None).into(),
            ],
            true,
        )
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
    async fn parse_input(&self, input: Value) -> Result<Self::Input, Box<dyn Error + Send + Sync>>;

    fn usage_limit(&self) -> Option<usize> {
        None
    }
}

#[derive(new)]
#[repr(transparent)]
pub struct ToolWrapper<T>
where
    T: ToolFunction,
{
    tool: T,
}

#[async_trait]
impl<T> Tool for ToolWrapper<T>
where
    T: ToolFunction,
{
    fn name(&self) -> String {
        self.tool.name()
    }

    fn description(&self) -> String {
        self.tool.description()
    }

    fn parameters(&self) -> ObjectField {
        self.tool.parameters()
    }

    fn strict(&self) -> bool {
        self.tool.strict()
    }

    async fn call(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
        let input = self.tool.parse_input(input).await?;
        let result = self.tool.run(input).await?;

        Ok(result.to_string())
    }

    fn usage_limit(&self) -> Option<usize> {
        self.tool.usage_limit()
    }
}

pub fn map_tools(tools: Vec<Arc<dyn Tool>>) -> HashMap<String, Arc<dyn Tool>> {
    tools
        .into_iter()
        .map(|tool| (tool.name().to_lowercase().replace(" ", "_"), tool))
        .collect()
}
