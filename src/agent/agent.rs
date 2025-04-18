use std::error::Error;

use async_trait::async_trait;

use crate::{
    diary::DiaryStep,
    schemas::{AgentResult, InputVariables},
    tools::Tool,
};

use super::AgentError;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(
        &self,
        steps: &[DiaryStep],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError>;

    async fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>>;
}
