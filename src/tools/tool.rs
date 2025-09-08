use crate::tools::{FunctionTool, McpTool};

pub enum Tool {
    Function(Box<dyn FunctionTool>),
    Mcp(McpTool),
}

// Executor will turn this into function tool or mcp tool spec, respectively
