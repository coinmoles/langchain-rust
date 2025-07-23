#[derive(Debug, Clone)]
pub enum ToolOutput {
    Text(String),
    List(Vec<String>),
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
        }
    }
}

impl From<String> for ToolOutput {
    fn from(value: String) -> Self {
        ToolOutput::Text(value)
    }
}

impl<T> From<Vec<T>> for ToolOutput
where
    ToolOutput: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let list = value
            .into_iter()
            .flat_map(|v| match ToolOutput::from(v) {
                ToolOutput::Text(text) => vec![text],
                ToolOutput::List(list) => list,
            })
            .collect();
        ToolOutput::List(list)
    }
}
