use crate::{agent::AgentError, schemas::AgentEvent, tools::Tool};

pub trait Instructor: Send + Sync {
    fn create_suffix(&self, tools: &[&dyn Tool]) -> String;
    fn parse_output(&self, output: &str) -> Result<AgentEvent, AgentError>;
}
