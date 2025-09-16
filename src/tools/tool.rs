use crate::tools::{FunctionTool, McpTool};

pub enum Tool<'a> {
    Function(Box<dyn FunctionTool + 'a>),
    Mcp(McpTool),
}

// Executor will turn this into function tool or mcp tool spec, respectively

impl Tool<'_> {
    pub fn name(&self) -> String {
        match self {
            Tool::Function(tool) => tool.name(),
            Tool::Mcp(tool) => tool.name.clone(),
        }
    }
}

impl<'a, F: FunctionTool + 'a> From<F> for Tool<'a> {
    fn from(tool: F) -> Self {
        Tool::Function(Box::new(tool))
    }
}

impl From<McpTool> for Tool<'_> {
    fn from(tool: McpTool) -> Self {
        Tool::Mcp(tool)
    }
}
