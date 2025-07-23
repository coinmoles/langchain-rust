pub struct ToolOutput {
    pub data: ToolData,
    pub summary: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ToolData {
    Text(String),
    List(Vec<String>),
}

impl std::fmt::Display for ToolData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolData::Text(text) => write!(f, "{text}"),
            ToolData::List(list) => {
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

impl From<String> for ToolData {
    fn from(value: String) -> Self {
        ToolData::Text(value)
    }
}

impl<T> From<Vec<T>> for ToolData
where
    ToolData: From<T>,
{
    fn from(value: Vec<T>) -> Self {
        let list = value
            .into_iter()
            .flat_map(|v| match ToolData::from(v) {
                ToolData::Text(text) => vec![text],
                ToolData::List(list) => list,
            })
            .collect();
        ToolData::List(list)
    }
}
