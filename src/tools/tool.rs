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
