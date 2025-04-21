use std::error::Error;

use async_trait::async_trait;
use langchain_rust::{
    agent::{Agent, OpenAiToolAgentBuilder},
    chain::Chain,
    llm::openai::OpenAI,
    memory::SimpleMemory,
    schemas::InputVariables,
    text_replacements,
    tools::{CommandExecutor, DuckDuckGoSearch, SerpApi, Tool, ToolFunction},
};

use serde_json::Value;

#[derive(Default)]
struct Date {}

#[async_trait]
impl ToolFunction for Date {
    type Input = ();
    type Result = String;

    fn name(&self) -> String {
        "Date".to_string()
    }

    fn description(&self) -> String {
        "Useful when you need to get the date, input should be an empty object ({})".to_string()
    }

    async fn parse_input(&self, _input: Value) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    async fn run(&self, _input: ()) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("25 of november of 2025".to_string())
    }
}

#[tokio::main]
async fn main() {
    let llm = OpenAI::default();
    let memory = SimpleMemory::new();
    let serpapi_tool = SerpApi::default();
    let duckduckgo_tool = DuckDuckGoSearch::default();
    let tool_calc = Date::default();
    let command_executor = CommandExecutor::default();
    let agent = OpenAiToolAgentBuilder::new()
        .tools([
            Box::new(serpapi_tool) as Box<dyn Tool>,
            tool_calc.into(),
            command_executor.into(),
            duckduckgo_tool.into(),
        ])
        .build(llm)
        .await
        .unwrap();

    let executor = agent.executor().with_memory(memory.into());

    let mut input_variables: InputVariables = text_replacements! {
        "input" => "What the name of the current dir, And what date is today",
    }
    .into();

    match executor.invoke(&mut input_variables).await {
        Ok(result) => {
            println!("Result: {:?}", result.replace("\n", " "));
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
