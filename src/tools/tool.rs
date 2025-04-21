use std::error::Error;
use std::string::String;

use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs,
};
use async_trait::async_trait;
use indoc::indoc;
use serde_json::Value;

use crate::tools::tool_field::StringField;

use super::tool_field::ToolParameters;

#[async_trait]
pub trait Tool: Send + Sync {
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
