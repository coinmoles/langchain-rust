use std::fmt::Display;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    agent::{Agent, AgentInput, DefaultStrategy, ExecutionContext, Strategy},
    chain::{Chain, ChainError, ChainOutput, InputCtor, OutputCtor},
    memory::Memory,
    schemas::{GetPrompt, Prompt, WithUsage},
    template::TemplateError,
};

use super::ExecutorOptions;

pub struct AgentExecutor<'agent, I: InputCtor, O: OutputCtor>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) agent: Box<dyn Agent<I, O> + 'agent>,
    pub(super) memory: Option<Arc<RwLock<dyn Memory>>>,
    pub(super) options: ExecutorOptions,
}

impl<'agent, I: InputCtor, O: OutputCtor> AgentExecutor<'agent, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn from_agent(agent: impl Agent<I, O> + 'agent) -> Self {
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

    pub fn with_options(mut self, options: ExecutorOptions) -> Self {
        self.options = options;
        self
    }

    pub fn execution<'exec, 'input, S: Strategy>(
        &'exec self,
        input: I::Target<'input>,
    ) -> ExecutionContext<'exec, 'agent, 'input, I, O, S> {
        ExecutionContext::new(self, input)
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Chain<I, O> for AgentExecutor<'_, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let output = self.execution::<DefaultStrategy>(input).start().await?;
        Ok(output.without_extra())
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<I::Target<'_>> for AgentExecutor<'_, I, O>
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, TemplateError> {
        self.agent.get_prompt(&AgentInput::new(input))
    }
}
