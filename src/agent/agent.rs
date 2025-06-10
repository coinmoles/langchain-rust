use std::fmt::Display;

use async_trait::async_trait;

use crate::{
    schemas::{AgentEvent, AgentStep, ChainOutput, InputCtor, OutputCtor, Prompt, WithUsage},
    template::TemplateError,
    tools::Tool,
};

use super::{AgentError, AgentExecutor, AgentInput};

#[async_trait]
pub trait Agent: Send + Sync {
    type InputCtor: InputCtor;
    type OutputCtor: OutputCtor;

    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<<Self::InputCtor as InputCtor>::Target<'i>>,
    ) -> Result<WithUsage<AgentEvent>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn executor<'a>(self) -> AgentExecutor<'a, Self::InputCtor, Self::OutputCtor>
    where
        Self: Sized + 'a,
        for<'any> <Self::InputCtor as InputCtor>::Target<'any>: Display,
        for<'any> <Self::OutputCtor as OutputCtor>::Target<'any>:
            ChainOutput<<Self::InputCtor as InputCtor>::Target<'any>>,
    {
        AgentExecutor::from_agent(self)
    }

    fn get_prompt(
        &self,
        input: &AgentInput<<Self::InputCtor as InputCtor>::Target<'_>>,
    ) -> Result<Prompt, TemplateError>;
}
