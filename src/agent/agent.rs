use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    chain::ChainError,
    schemas::{AgentEvent, AgentStep, ChainInputCtor, ChainOutput, Prompt, WithUsage},
    tools::Tool,
};

use super::{AgentError, AgentExecutor, AgentInput};

#[async_trait]
pub trait Agent: Send + Sync {
    type InputCtor: ChainInputCtor;
    type Output: ChainOutput;

    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<WithUsage<AgentEvent>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn get_prompt<'i>(
        &self,
        input: AgentInput<'i, <Self::InputCtor as ChainInputCtor>::Target<'i>>,
    ) -> Result<Prompt, ChainError>;

    fn executor<'a>(self) -> AgentExecutor<'a, Self::InputCtor, Self::Output>
    where
        Self: Sized + 'a,
        for<'b> <Self::InputCtor as ChainInputCtor>::Target<'b>: Display,
    {
        AgentExecutor::from_agent(self)
    }
}
