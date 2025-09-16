use std::collections::HashMap;

use crate::{tools::FunctionTool, utils::helper::normalize_tool_name};

use super::Toolbox;

pub struct SimpleToolbox {
    name: String,
    tools: HashMap<String, Box<dyn FunctionTool>>,
}

impl SimpleToolbox {
    pub fn new(name: impl Into<String>, tools: HashMap<String, Box<dyn FunctionTool>>) -> Self {
        Self {
            name: name.into(),
            tools,
        }
    }

    pub fn from_tools(
        name: impl Into<String>,
        tools: impl IntoIterator<Item = Box<dyn FunctionTool>>,
    ) -> Self {
        let tools = tools
            .into_iter()
            .map(|tool| (normalize_tool_name(&tool.name()), tool))
            .collect::<HashMap<_, _>>();
        Self {
            name: name.into(),
            tools,
        }
    }
}

impl Toolbox for SimpleToolbox {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_tools(&self) -> HashMap<&str, &dyn FunctionTool> {
        self.tools
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref()))
            .collect()
    }
}
