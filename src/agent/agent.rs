use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    chain::ChainError,
    schemas::{AgentEvent, AgentStep, Ctor, InputCtor, Prompt, WithUsage},
    tools::Tool,
};

use super::{AgentError, AgentExecutor, AgentInput};

#[async_trait]
pub trait Agent: Send + Sync {
    type InputCtor: InputCtor;
    type OutputCtor: Ctor;

    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<<Self::InputCtor as InputCtor>::Target<'i>>,
    ) -> Result<WithUsage<AgentEvent>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn get_prompt<'i>(
        &self,
        input: AgentInput<<Self::InputCtor as InputCtor>::Target<'i>>,
    ) -> Result<Prompt, ChainError>;

    fn executor<'a>(self) -> AgentExecutor<'a, Self::InputCtor, Self::OutputCtor>
    where
        Self: Sized + 'a,
        for<'b> <Self::InputCtor as InputCtor>::Target<'b>: Display,
    {
        AgentExecutor::from_agent(self)
    }
}
