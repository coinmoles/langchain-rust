use std::fmt::{self, Display};

use super::{GenerateResultContent, TokenUsage};

#[derive(Debug, Clone, Default)]
pub struct GenerateResult {
    pub content: GenerateResultContent,
    pub usage: Option<TokenUsage>,
}

impl GenerateResult {
    pub fn new(content: GenerateResultContent, usage: Option<TokenUsage>) -> Self {
        Self { content, usage }
    }
}

impl Display for GenerateResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.content {
            GenerateResultContent::Text(text) => write!(f, "{}", text)?,
            GenerateResultContent::ToolCall(tool_calls) => {
                writeln!(f, "Strucuted tool call:")?;
                for tool_call in tool_calls {
                    write!(f, "{}", tool_call)?;
                }
            }
            GenerateResultContent::Refusal(refusal) => write!(f, "Refused: {}", refusal)?,
        };

        if let Some(usage) = &self.usage {
            write!(f, "\n\n{}", usage)?
        }

        Ok(())
    }
}
