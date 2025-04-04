use std::{error::Error, sync::Arc};

use async_trait::async_trait;

use crate::{
    schemas::{AgentResult, InputVariables, ToolCall},
    tools::Tool,
};

use super::AgentError;

#[async_trait]
pub trait Agent: Send + Sync {
    async fn plan(
        &self,
        intermediate_steps: &[(ToolCall, String)],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<Arc<dyn Tool>>;

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>>;
}
