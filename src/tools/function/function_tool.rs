use std::string::String;

use async_trait::async_trait;
use schemars::{schema_for, Schema};
use serde_json::Value;

use crate::{
    schemas::FunctionSpec,
    tools::{Function, ToolData, ToolError, ToolOutput},
};

use super::function_input::DefaultFunctionInput;

pub(crate) mod sealed {
    /// A sealed trait to prevent external implementations of the `ToolInternal` trait.
    pub trait Sealed {}
}

/// A dyn-compatible, generic-less interface for tools.
///
/// This trait is "sealed", meaning it cannot be implemented outside of this module.
/// This trait should only be implemented via a blanket impl, which automatically implements this trait for any type that implements `Tool`.
#[async_trait]
pub trait FunctionTool: sealed::Sealed + Send + Sync {
    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// JSON schema for the tool input parameters.
    ///
    /// Used for OpenAI function call.
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
    fn parameters(&self) -> Schema {
        schema_for!(DefaultFunctionInput)
    }

    /// Value for `strict` in the OpenAI function call
    ///
    /// If not implemented, it will default to `false`
    fn strict(&self) -> bool {
        false
    }

    /// Processes an input string and executes the tool's functionality, returning a `Result`.
    async fn call(&self, input: Value) -> Result<ToolOutput, ToolError>;

    fn usage_limit(&self) -> Option<usize> {
        None
    }

    fn get_spec(&self) -> FunctionSpec {
        FunctionSpec::new(
            self.name(),
            Some(self.description()),
            self.parameters(),
            self.strict(),
        )
    }
}

impl<T> sealed::Sealed for T where T: Function {}

#[async_trait]
impl<T> FunctionTool for T
where
    T: Function + sealed::Sealed,
{
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description()
    }

    fn parameters(&self) -> Schema {
        self.parameters()
    }

    fn strict(&self) -> bool {
        self.strict()
    }

    async fn call(&self, input: Value) -> Result<ToolOutput, ToolError> {
        let input: T::Input = self.parse_input(input).await?;
        let input_summary = self.summarize_input(&input);
        let result: T::Output = self.call(input).await.map_err(ToolError::ExecutionError)?;
        let output_summary = self.summarize_output(&result);

        let data: ToolData = result.into();
        let summary = match (input_summary, output_summary) {
            (Some(input), Some(output)) => Some(format!("{input}{output}")),
            (Some(input), None) => Some(input),
            (None, Some(output)) => Some(output),
            (None, None) => None,
        };

        Ok(ToolOutput { data, summary })
    }

    fn usage_limit(&self) -> Option<usize> {
        self.usage_limit()
    }
}

impl<'a, T> From<T> for Box<dyn FunctionTool + 'a>
where
    T: FunctionTool + 'a,
{
    fn from(val: T) -> Self {
        Box::new(val)
    }
}
