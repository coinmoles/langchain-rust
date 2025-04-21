use std::error::Error;

use async_trait::async_trait;

use crate::{
    schemas::{AgentResult, AgentStep, InputVariables},
    tools::Tool,
};

use super::{AgentError, AgentExecutor};

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(
        &self,
        steps: &[AgentStep],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError>;

    async fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>>;

    fn executor<'a>(self) -> AgentExecutor<'a>
    where
        Self: Sized + 'a,
    {
        AgentExecutor::from_agent(self)
    }
}
