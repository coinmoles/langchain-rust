use std::collections::HashMap;

use crate::tools::FunctionTool;

use super::Toolbox;

pub struct SimpleToolbox {
    name: String,
    tools: HashMap<String, Box<dyn FunctionTool>>,
}

impl SimpleToolbox {
    pub fn new<S>(name: S, tools: HashMap<String, Box<dyn FunctionTool>>) -> Self
    where
        S: Into<String>,
    {
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
