use std::{collections::HashMap, fmt::Display, sync::Arc};

use async_trait::async_trait;

use crate::{
    agent::{
        Agent, AgentError, AgentInput, AgentInputCtor, AgentOutput, AgentOutputCtor, AgentStep,
    },
    chain::LLMChain,
    schemas::{
        ChainOutput, DefaultChainInputCtor, GetPrompt, InputCtor, Message, OutputCtor, Prompt,
        StringCtor, WithUsage,
    },
    template::TemplateError,
    tools::{Tool, Toolbox},
};

use super::ConversationalAgentBuilder;

pub struct ConversationalAgent<I = DefaultChainInputCtor, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
    pub(super) tools: HashMap<String, Box<dyn Tool>>,
    pub(super) toolboxes: Vec<Arc<dyn Toolbox>>, // Has to be Arc because ownership needs to be shared with ListTools
    pub(super) _phantom: std::marker::PhantomData<O>,
}

impl<I, O> ConversationalAgent<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn new(
        llm_chain: LLMChain<AgentInputCtor<I>, AgentOutputCtor>,
        tools: HashMap<String, Box<dyn Tool>>,
        toolboxes: Vec<Arc<dyn Toolbox>>,
    ) -> Self {
        Self {
            llm_chain,
            tools,
            toolboxes,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn builder<'a, 'b>() -> ConversationalAgentBuilder<'a, 'b, I, O> {
        ConversationalAgentBuilder::new()
    }

    fn construct_scratchpad(&self, intermediate_steps: &[AgentStep]) -> Vec<Message> {
        intermediate_steps
            .iter()
            .flat_map(|step| {
                vec![
                    Message::new_ai_message(&step.tool_call),
                    Message::new_human_message(&step.result),
                ]
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl<I, O> Agent for ConversationalAgent<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn plan<'i>(
        &self,
        steps: &[AgentStep],
        input: &mut AgentInput<I::Target<'i>>,
    ) -> Result<WithUsage<AgentOutput>, AgentError> {
        input.set_agent_scratchpad(self.construct_scratchpad(steps));
        let result = self.llm_chain.call_with_reference(input).await?;
        Ok(result)
    }

    fn get_tool(&self, tool_name: &str) -> Option<&dyn Tool> {
        if let Some(tool) = self.tools.get(tool_name).map(|t| t.as_ref()) {
            return Some(tool);
        }

        for toolbox in &self.toolboxes {
            if let Ok(tool) = toolbox.get_tool(tool_name) {
                return Some(tool);
            }
        }

        None
    }

    fn get_prompt(
        &self,
        input: &AgentInput<<Self::InputCtor as InputCtor>::Target<'_>>,
    ) -> Result<Prompt, TemplateError> {
        self.llm_chain.get_prompt(input)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use async_openai::config::OpenAIConfig;
    use async_trait::async_trait;
    use serde_json::Value;

    use crate::{
        agent::{Agent, ConversationalAgent},
        chain::Chain,
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        schemas::DefaultChainInput,
        tools::ToolFunction,
    };

    #[derive(Default)]
    struct Calc {}

    #[async_trait]
    impl ToolFunction for Calc {
        type Input = String;
        type Result = i128;

        fn name(&self) -> String {
            "Calculator".to_string()
        }
        fn description(&self) -> String {
            "Usefull to make calculations".to_string()
        }
        async fn parse_input(&self, input: Value) -> Result<String, serde_json::Error> {
            Ok(input.to_string())
        }
        async fn run(&self, _input: String) -> Result<i128, Box<dyn Error + Send + Sync>> {
            Ok(25)
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_invoke_agent() {
        let llm: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt4.to_string())
            .build();
        let memory = SimpleMemory::new();
        let tool_calc = Calc::default();
        let agent: ConversationalAgent = ConversationalAgent::builder()
            .tools([tool_calc])
            .build(llm)
            .await
            .unwrap();
        let input = DefaultChainInput::new(
            "hola,Me llamo luis, y tengo 10 anos, y estudio Computer scinence",
        );
        let executor = agent.executor().with_memory(memory.into());
        match executor.call(input).await {
            Ok(result) => {
                println!("Result: {:?}", result.content);
            }
            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }

        let input = DefaultChainInput::new("cuanta es la edad de luis +10 y que estudia");
        match executor.call(input).await {
            Ok(result) => println!("Result: {:?}", result.content),

            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }
    }
}
