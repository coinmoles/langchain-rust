use std::error::Error;

use async_trait::async_trait;
use langchain_rust::{
    agent::Agent,
    chain::{Chain, DefaultChainInput, DefaultChainInputCtor},
    llm::OpenAIChat,
    memory::SimpleMemory,
    tools::{CommandExecutor, DuckDuckGoSearch, Function, SerpApi},
};

use serde_json::Value;

#[derive(Default)]
struct Date {}

#[async_trait]
impl Function for Date {
    type Input = ();
    type Output = String;

    fn name(&self) -> String {
        "Date".to_string()
    }

    fn description(&self) -> String {
        "Useful when you need to get the date, input should be an empty object ({})".to_string()
    }

    async fn parse_input(&self, _input: Value) -> Result<(), serde_json::Error> {
        Ok(())
    }

    async fn call(&self, _input: ()) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("25 of november of 2025".to_string())
    }
}

#[tokio::main]
async fn main() {
    let llm = OpenAIChat::default();
    let memory = SimpleMemory::new();
    let agent: Agent<DefaultChainInputCtor> = Agent::builder()
        .tools([
            SerpApi::default().into(),
            Date::default().into(),
            DuckDuckGoSearch::default().into(),
            CommandExecutor::default().into(),
        ])
        .build(llm);

    let executor = agent.executor().with_memory(memory.into());

    let input_variables =
        DefaultChainInput::new("What the name of the current dir, And what date is today");

    match executor.call(input_variables).await {
        Ok(result) => {
            println!("Result: {:?}", result.content.replace("\n", " "));
        }
        Err(e) => panic!("Error invoking LLMChain: {e:?}"),
    }
}
