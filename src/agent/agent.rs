use std::{collections::HashSet, error::Error};

use async_trait::async_trait;

use crate::{
    agent::DEFAULT_OUTPUT_KEY,
    schemas::{AgentEvent, AgentStep, InputVariableCtor, OutputVariable, Prompt, WithUsage},
    tools::Tool,
};

use super::{AgentError, AgentExecutor, AgentInput, DEFAULT_INPUT_KEY};

#[async_trait]
pub trait Agent: Send + Sync {
    type InputCtor: InputVariableCtor;
    type Output: OutputVariable;

    async fn plan<'b>(
        &self,
        steps: &[AgentStep],
        inputs: AgentInput<'b, <Self::InputCtor as InputVariableCtor>::InputVariable<'b>>,
    ) -> Result<WithUsage<AgentEvent>, AgentError>;

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool>;

    fn get_prompt<'b>(
        &self,
        inputs: &AgentInput<'b, <Self::InputCtor as InputVariableCtor>::InputVariable<'b>>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>;

    fn get_input_keys(&self) -> HashSet<&str> {
        HashSet::from_iter([DEFAULT_INPUT_KEY])
    }

    fn get_output_keys(&self) -> Vec<&str> {
        vec![DEFAULT_OUTPUT_KEY]
    }

    fn executor<'a>(self) -> AgentExecutor<'a, Self::InputCtor, Self::Output>
    where
        Self: Sized + 'a,
    {
        AgentExecutor::from_agent(self)
    }
}
