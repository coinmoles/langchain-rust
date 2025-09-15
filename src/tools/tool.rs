use crate::tools::{FunctionTool, McpTool};

pub enum Tool {
    Function(Box<dyn FunctionTool>),
    Mcp(McpTool),
}

// Executor will turn this into function tool or mcp tool spec, respectively

impl Tool {
    pub fn name(&self) -> String {
        match self {
            Tool::Function(tool) => tool.name(),
            Tool::Mcp(tool) => tool.name.clone(),
        }
    }
}

impl<F: FunctionTool + 'static> From<F> for Tool {
    fn from(tool: F) -> Self {
        Tool::Function(Box::new(tool))
    }
}

impl From<McpTool> for Tool {
    fn from(tool: McpTool) -> Self {
        Tool::Mcp(tool)
    }
}
