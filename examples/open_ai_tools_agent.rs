use std::error::Error;

use async_trait::async_trait;
use langchain_rust::{
    agent::{Agent, OpenAiToolAgent},
    chain::Chain,
    llm::openai::OpenAI,
    memory::SimpleMemory,
    schemas::{DefaultChainInput, DefaultChainInputCtor},
    tools::{CommandExecutor, DuckDuckGoSearch, SerpApi, Tool, ToolFunction},
    tools_vec,
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

    async fn parse_input(&self, _input: Value) -> Result<(), serde_json::Error> {
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
    let agent: OpenAiToolAgent<DefaultChainInputCtor> = OpenAiToolAgent::builder()
        .tools(tools_vec![
            SerpApi::default(),
            Date::default(),
            DuckDuckGoSearch::default(),
            CommandExecutor::default(),
        ])
        .build(llm)
        .await
        .unwrap();

    let executor = agent.executor().with_memory(memory.into());

    let input_variables =
        DefaultChainInput::new("What the name of the current dir, And what date is today");

    match executor.call(input_variables).await {
        Ok(result) => {
            println!("Result: {:?}", result.content.replace("\n", " "));
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
