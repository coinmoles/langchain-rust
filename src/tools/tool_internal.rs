use std::string::String;

use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs,
};
use async_trait::async_trait;
use indoc::formatdoc;
use schemars::{schema::RootSchema, schema_for};
use serde_json::Value;

use crate::{tools::ToolFunction, utils::helper::normalize_tool_name};

use super::{describe_parameters, tool_input::DefaultToolInput, ToolError};

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
    fn parameters(&self) -> RootSchema {
        schema_for!(DefaultToolInput)
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
    async fn call(&self, input: Value) -> Result<String, ToolError>;

    fn usage_limit(&self) -> Option<usize> {
        None
    }

    fn to_plain_description(&self) -> String {
        let name_and_desc = format!(
            "> {}: {}",
            normalize_tool_name(&self.name()),
            self.description()
        );
        let parameters = describe_parameters(&self.parameters());

        match parameters {
            Ok(parameters) => formatdoc! {"
                {name_and_desc}
                <INPUT_FORMAT>
                {parameters}
                </INPUT_FORMAT>"},
            Err(e) => {
                log::warn!(
                    "Failed to describe parameters for tool {}: {e}",
                    self.name(),
                );
                name_and_desc
            }
        }
    }

    fn as_openai_tool(&self) -> ChatCompletionTool {
        let parameters = serde_json::to_value(self.parameters()).unwrap_or_else(|e| {
            log::warn!(
                "Failed to serialize parameters for tool {}: {e}",
                self.name(),
            );
            Value::Null
        });

        let tool = FunctionObjectArgs::default()
            .name(self.name().to_lowercase().replace(" ", "_"))
            .description(self.description())
            .parameters(parameters)
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

    fn parameters(&self) -> RootSchema {
        self.parameters()
    }

    fn strict(&self) -> bool {
        self.strict()
    }

    async fn call(&self, input: Value) -> Result<String, ToolError> {
        let input = self.parse_input(input).await?;
        let result = self.run(input).await.map_err(ToolError::ExecutionError)?;
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
