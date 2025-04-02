use std::error::Error;
use std::string::String;

use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType, FunctionObjectArgs,
};
use async_trait::async_trait;
use indoc::indoc;
use serde_json::Value;

use crate::tools::tool_field::{ObjectField, StringField};

use super::tool_field::ToolField;

#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// This are the parametters for OpenAi-like function call.
    /// You should return a jsnon like this one
    /// ```json
    /// {
    ///     "type": "object",
    ///     "properties": {
    ///         "command": {
    ///             "type": "string",
    ///             "description": "The raw command you want executed"
    ///                 }
    ///     },
    ///     "required": ["command"]
    /// }
    ///
    /// If there s no implementation the defaul will be the self.description()
    ///```
    fn parameters(&self) -> ObjectField {
        ObjectField::new_tool_input(vec![StringField::new(
            "input",
            Some("The input for the tool".into()),
            true,
            None,
        )
        .into()])
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
            The input for this tool MUST be in the following format:
            {}"},
            self.name(),
            self.description(),
            self.parameters().properties_description()
        )
    }

    fn try_into_opeai_tool(&self) -> Result<ChatCompletionTool, OpenAIError> {
        let tool = FunctionObjectArgs::default()
            .name(self.name().replace(" ", "_"))
            .description(self.description())
            .parameters(self.parameters().to_openai_field())
            .build()?;

        ChatCompletionToolArgs::default()
            .r#type(ChatCompletionToolType::Function)
            .function(tool)
            .build()
    }
}
