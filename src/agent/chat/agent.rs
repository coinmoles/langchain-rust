use std::{collections::HashMap, error::Error, sync::Arc};

use async_trait::async_trait;

use crate::{
    agent::{instructor::Instructor, Agent, AgentError},
    chain::chain_trait::Chain,
    schemas::{AgentResult, AgentStep, InputVariables, Message, Prompt},
    tools::{Tool, Toolbox},
};

pub struct ConversationalAgent {
    pub(crate) chain: Box<dyn Chain>,
    pub(crate) tools: HashMap<String, Box<dyn Tool>>,
    pub(crate) toolboxes: Vec<Arc<dyn Toolbox>>, // Has to be Arc because ownership needs to be shared with ListTools
    pub(crate) instructor: Box<dyn Instructor>,
}

impl ConversationalAgent {
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
impl Agent for ConversationalAgent {
    async fn plan(
        &self,
        intermediate_steps: &[AgentStep],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError> {
        let scratchpad = self.construct_scratchpad(intermediate_steps);
        inputs.insert_placeholder_replacement("agent_scratchpad", scratchpad);
        let output = self.chain.call(inputs).await?;

        let content = self.instructor.parse_output(output.content.text())?;
        let usage = output.usage;

        Ok(AgentResult { content, usage })
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

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error>> {
        self.chain.get_prompt(inputs)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use async_openai::config::OpenAIConfig;
    use async_trait::async_trait;
    use serde_json::Value;

    use crate::{
        agent::{chat::builder::ConversationalAgentBuilder, Agent},
        chain::chain_trait::Chain,
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        schemas::InputVariables,
        text_replacements,
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
        async fn parse_input(&self, input: Value) -> Result<String, Box<dyn Error + Send + Sync>> {
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
        let agent = ConversationalAgentBuilder::new()
            .tools([tool_calc])
            .build(llm)
            .await
            .unwrap();
        let mut input_variables: InputVariables = text_replacements! {
            "input" => "hola,Me llamo luis, y tengo 10 anos, y estudio Computer scinence",
        }
        .into();
        let executor = agent.executor().with_memory(memory.into());
        match executor.invoke(&mut input_variables).await {
            Ok(result) => {
                println!("Result: {:?}", result);
            }
            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }
        let mut input_variables: InputVariables = text_replacements! {
            "input" => "cuanta es la edad de luis +10 y que estudia",
        }
        .into();
        match executor.invoke(&mut input_variables).await {
            Ok(result) => {
                println!("Result: {:?}", result);
            }
            Err(e) => panic!("Error invoking LLMChain: {:?}", e),
        }
    }
}
