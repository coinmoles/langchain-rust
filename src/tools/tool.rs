use std::string::String;
use std::{error::Error, fmt::Display};

use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs,
};
use async_trait::async_trait;
use indoc::indoc;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::tools::tool_field::StringField;

use super::tool_field::ToolParameters;

mod sealed {
    /// A sealed trait to prevent external implementations of the `Tool` trait.
    ///
    /// To create your own tool, you must implement ToolFunction, which will automatically implement Tool via blanket impl.
    pub trait Sealed {}
}

#[async_trait]
pub trait Tool: sealed::Sealed + Send + Sync {
    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// Parameters for OpenAI function call.
    ///
    /// If not implemented, it will default to
    /// ```json
    /// {
    ///     "input": {
    ///         "type": "string",
    ///         "description": "The input for the tool"
    ///     },
    ///     required: ["input"]
    /// }
    /// ```
    fn parameters(&self) -> ToolParameters {
        ToolParameters::new([StringField::new("input")
            .description("The input for the tool")
            .into()])
        .additional_properties(false)
    }

    /// Value for `strict` in the OpenAI function call
    ///
    /// If not implemented, it will default to `false`
    fn strict(&self) -> bool {
        false
    }

    /// Processes an input string and executes the tool's functionality, returning a `Result`.
    ///
    /// This function utilizes `parse_input` to parse the input and then calls `run`.
    /// Its used by the Agent
    async fn call(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>>;

    fn usage_limit(&self) -> Option<usize> {
        None
    }

    fn to_plain_description(&self) -> String {
        format!(
            indoc! {"
            > {}: {}
            <INPUT_FORMAT>
            {}
            </INPUT_FORMAT>"},
            self.name().to_lowercase().replace(" ", "_"),
            self.description(),
            self.parameters().properties_description()
        )
    }

    fn into_openai_tool(&self) -> ChatCompletionTool {
        let tool = FunctionObjectArgs::default()
            .name(self.name().to_lowercase().replace(" ", "_"))
            .description(self.description())
            .parameters(self.parameters().to_openai_field())
            .strict(self.strict())
            .build()
            .unwrap_or_else(|e| unreachable!("All fields must be set: {}", e));

        ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(tool)
            .build()
            .unwrap_or_else(|e| unreachable!("All fields must be set: {}", e))
    }
}

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

impl<T> sealed::Sealed for T where T: ToolFunction {}

#[async_trait]
impl<T> Tool for T
where
    T: ToolFunction + sealed::Sealed,
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

impl<'a, T> From<T> for Box<dyn Tool + 'a>
where
    T: Tool + 'a,
{
    fn from(val: T) -> Self {
        Box::new(val)
    }
}

#[macro_export]
macro_rules! tools_vec {
    ($($tool:expr),* $(,)?) => {
        vec![$(Box::new($tool) as Box<dyn Tool>),*]
    };
}
