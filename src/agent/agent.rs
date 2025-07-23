use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    agent::{AgentOutput, AgentStep},
    chain::{ChainOutput, InputCtor, OutputCtor},
    schemas::{Prompt, WithUsage},
    template::TemplateError,
    tools::ToolInternal,
};

use super::{AgentError, AgentExecutor, AgentInput};

#[async_trait]
pub trait Agent<I: InputCtor, O: OutputCtor>: Send + Sync
where
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    async fn plan<'a>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<I::Target<'a>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolInternal>;

    fn executor<'a>(self) -> AgentExecutor<'a, I, O>
    where
        Self: Sized + 'a,
    {
        AgentExecutor::from_agent(self)
    }

    fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError>;
}
