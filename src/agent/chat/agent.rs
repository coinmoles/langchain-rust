use std::{collections::HashMap, error::Error};

use async_trait::async_trait;

use crate::{
    agent::{agent::Agent, AgentError},
    chain::chain_trait::Chain,
    prompt_template,
    schemas::{AgentResult, InputVariables, Message, MessageType, ToolCall},
    template::{MessageOrTemplate, MessageTemplate, PromptTemplate},
    text_replacements,
    tools::Tool,
};

use super::{parse::parse_agent_output, prompt::SUFFIX};

pub struct ConversationalAgent {
    pub(crate) chain: Box<dyn Chain>,
    pub(crate) tools: HashMap<String, Box<dyn Tool>>,
}

impl ConversationalAgent {
    pub fn create_prompt(
        system_prompt: &str,
        initial_prompt: &str,
        tools: &HashMap<String, Box<dyn Tool>>,
    ) -> Result<PromptTemplate, AgentError> {
        let tool_names = tools.keys().cloned().collect::<Vec<_>>().join(", ");
        let tool_string = tools
            .values()
            .map(|tool| tool.to_plain_description())
            .collect::<Vec<_>>()
            .join("\n");
        let input_variables_fstring: InputVariables = text_replacements! {
            "tool_names" => tool_names,
            "tools" => tool_string,
        }
        .into();

        let system_prompt = MessageTemplate::from_jinja2(
            MessageType::SystemMessage,
            &format!("{}{}", system_prompt, SUFFIX),
        )
        .format(&input_variables_fstring)?;

        let prompt = prompt_template![
            system_prompt,
            MessageOrTemplate::Placeholder("chat_history".into()),
            MessageTemplate::from_jinja2(MessageType::HumanMessage, initial_prompt),
            MessageOrTemplate::Placeholder("agent_scratchpad".into()),
            MessageOrTemplate::Placeholder("ultimatum".into())
        ];

        Ok(prompt)
    }

    fn construct_scratchpad(&self, intermediate_steps: &[(ToolCall, String)]) -> Vec<Message> {
        intermediate_steps
            .iter()
            .flat_map(|(tool_call, result)| {
                vec![
                    Message::new(MessageType::AIMessage, tool_call),
                    Message::new(MessageType::HumanMessage, result),
                ]
            })
            .collect::<Vec<_>>()
    }
}

#[async_trait]
impl Agent for ConversationalAgent {
    async fn plan(
        &self,
        intermediate_steps: &[(ToolCall, String)],
        inputs: &mut InputVariables,
    ) -> Result<AgentResult, AgentError> {
        let scratchpad = self.construct_scratchpad(intermediate_steps);
        inputs.insert_placeholder_replacement("agent_scratchpad", scratchpad);
        let output = self.chain.call(inputs).await?;

        let content = parse_agent_output(&output.content.text())?;
        let usage = output.usage;

        Ok(AgentResult { content, usage })
    }

    fn get_tool(&self, tool_name: &str) -> Option<&Box<dyn Tool>> {
        self.tools.get(tool_name)
    }

    fn log_messages(&self, inputs: &InputVariables) -> Result<(), Box<dyn Error>> {
        self.chain.log_messages(inputs)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use async_openai::config::OpenAIConfig;
    use async_trait::async_trait;
    use serde_json::Value;

    use crate::{
        agent::{chat::builder::ConversationalAgentBuilder, executor::AgentExecutor},
        chain::chain_trait::Chain,
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        schemas::InputVariables,
        text_replacements,
        tools::{map_tools, ToolFunction},
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
            .tools(map_tools(vec![tool_calc.into()]))
            .build(llm)
            .unwrap();
        let mut input_variables: InputVariables = text_replacements! {
            "input" => "hola,Me llamo luis, y tengo 10 anos, y estudio Computer scinence",
        }
        .into();
        let executor = AgentExecutor::from_agent(agent).with_memory(memory.into());
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
