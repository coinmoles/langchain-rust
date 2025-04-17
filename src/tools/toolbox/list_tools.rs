use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use async_trait::async_trait;

use crate::tools::{tool_field::ToolParameters, Tool};

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

    pub fn into_boxed_tool(self) -> Box<dyn Tool> {
        Box::new(self)
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
                    "{{ name: {}, description: {}, parameters: {}, strict: {} }}",
                    self.name(),
                    self.description(),
                    self.parameters().to_openai_field(),
                    self.strict()
                ),
            )
            .finish()
    }
}

#[async_trait]
impl<T> Tool for ListTools<T>
where
    T: Toolbox + ?Sized,
{
    fn name(&self) -> String {
        format!("List tools in {}", self.0.name())
    }

    fn description(&self) -> String {
        format!("List all tools in the toolbox {}", self.0.name())
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters::new([]).additional_properties(false)
    }

    async fn call(
        &self,
        _: serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let tools = self.0.get_tools().await?;
        let tool_descriptions: Vec<String> = tools
            .values()
            .map(|tool| tool.to_plain_description())
            .collect();
        Ok(tool_descriptions.join("\n---\n"))
    }
}
