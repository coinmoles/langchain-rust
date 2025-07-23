use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl std::fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolOutput::Text(text) => write!(f, "{text}"),
            ToolOutput::List(list) => {
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        writeln!(f, "\n---")?;
                    }
                    write!(f, "{item}")?;
                }
                Ok(())
            }
            ToolOutput::Map(map) => {
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        writeln!(f, "\n---")?;
                    }
                    write!(f, "{key}:\n{value}")?;
                }
                Ok(())
            }
        }
    }
}

impl From<String> for ToolOutput {
    fn from(value: String) -> Self {
        ToolOutput::Text(value)
    }
}

impl From<Vec<String>> for ToolOutput {
    fn from(value: Vec<String>) -> Self {
        ToolOutput::List(value)
    }
}

impl From<HashMap<String, String>> for ToolOutput {
    fn from(value: HashMap<String, String>) -> Self {
        ToolOutput::Map(value)
    }
}
