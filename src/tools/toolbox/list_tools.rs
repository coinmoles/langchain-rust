use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use async_trait::async_trait;
use serde_json::Value;

use crate::tools::ToolFunction;

use super::Toolbox;

pub struct ListTools<T>(Arc<T>)
where
    T: Toolbox + ?Sized;

impl<T> ListTools<T>
where
    T: Toolbox + ?Sized + 'static,
{
    pub fn new(toolbox: &Arc<T>) -> Self {
        Self(Arc::clone(toolbox))
    }
}

impl<T> Debug for ListTools<T>
where
    T: Toolbox + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolWrapper")
            .field(
                "tool",
                &format_args!(
                    "{{ name: {}, description: {}, parameters: {:#?}, strict: {} }}",
                    self.name(),
                    self.description(),
                    self.parameters(),
                    self.strict()
                ),
            )
            .finish()
    }
}

#[async_trait]
impl<T> ToolFunction for ListTools<T>
where
    T: Toolbox + ?Sized,
{
    type Input = ();
    type Result = String;

    fn name(&self) -> String {
        format!("List tools in {}", self.0.name())
    }

    fn description(&self) -> String {
        format!("List all tools in the toolbox {}", self.0.name())
    }

    async fn parse_input(&self, _input: Value) -> Result<Self::Input, serde_json::Error> {
        Ok(())
    }

    async fn run(&self, _: ()) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let tools = self.0.get_tools()?;
        let tool_descriptions: Vec<String> = tools
            .values()
            .map(|tool| tool.to_plain_description())
            .collect();
        Ok(tool_descriptions.join("\n---\n"))
    }
}
