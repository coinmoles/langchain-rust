use std::collections::HashMap;

use crate::tools::{ToolDyn, ToolError};

use super::Toolbox;

pub struct SimpleToolbox {
    name: String,
    tools: HashMap<String, Box<dyn ToolDyn>>,
}

impl SimpleToolbox {
    pub fn new<S>(name: S, tools: HashMap<String, Box<dyn ToolDyn>>) -> Self
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

    fn get_tools(&self) -> Result<HashMap<&str, &dyn ToolDyn>, ToolError> {
        Ok(self
            .tools
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref()))
            .collect())
    }
}
