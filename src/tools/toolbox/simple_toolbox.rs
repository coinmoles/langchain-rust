use std::collections::HashMap;

use crate::tools::Tool;

use super::Toolbox;

pub struct SimpleToolbox {
    name: String,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl SimpleToolbox {
    pub fn new<S>(name: S, tools: HashMap<String, Box<dyn Tool>>) -> Self
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

    fn get_tools(
        &self,
    ) -> Result<HashMap<&str, &dyn Tool>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .tools
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_ref()))
            .collect())
    }
}
