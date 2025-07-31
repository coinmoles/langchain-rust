use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    agent::{AgentOutput, AgentStep},
    chain::{ChainOutput, InputCtor, OutputCtor},
    schemas::{Message, Prompt, WithUsage},
    template::TemplateError,
    tools::ToolDyn,
};

use super::{AgentError, AgentExecutor, AgentInput};

#[async_trait]
pub trait Agent<I: InputCtor, O: OutputCtor>: Send + Sync {
    async fn construct_scratchpad(&self, steps: &[AgentStep]) -> Result<Vec<Message>, AgentError>;

    async fn plan<'a>(
        &self,
        input: &AgentInput<I::Target<'a>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn ToolDyn>;

    fn executor<'a>(self) -> AgentExecutor<'a, I, O>
    where
        Self: Sized + 'a,
        for<'any> I::Target<'any>: Display,
        for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
    {
        AgentExecutor::from_agent(self)
    }

    fn get_prompt(&self, input: &AgentInput<I::Target<'_>>) -> Result<Prompt, TemplateError>;
}
