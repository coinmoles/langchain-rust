use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{ExecutionContext, ExecutorOptions};
use crate::{
    agent::Agent,
    chain::{chain_trait::Chain, ChainError},
    memory::Memory,
    schemas::{GenerateResult, InputVariables, ToolCall},
};

type ExecutorStep = (ToolCall, String);

pub struct AgentExecutor {
    pub(super) agent: Box<dyn Agent>,
    pub(super) memory: Option<Arc<RwLock<dyn Memory>>>,
    pub(super) options: ExecutorOptions,
}

impl AgentExecutor {
    pub fn from_agent<A>(agent: A) -> Self
    where
        A: Agent + Send + Sync + 'static,
    {
        Self {
            agent: Box::new(agent),
            memory: None,
            options: ExecutorOptions::default(),
        }
    }

    pub fn with_memory(mut self, memory: Arc<RwLock<dyn Memory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn execution(&self) -> ExecutionContext {
        ExecutionContext::new(self)
    }
}

#[async_trait]
impl Chain for AgentExecutor {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        self.execution().start(input_variables).await
    }

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>> {
        self.agent.log_messages(inputs)
    }
}
