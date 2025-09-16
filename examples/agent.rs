use async_openai::config::OpenAIConfig;
use langchain_rust::agent::Agent;
use langchain_rust::chain::{Chain, DefaultChainInput, DefaultChainInputCtor};
use langchain_rust::llm::{OpenAIChat, OpenAIModel};
use langchain_rust::memory::SimpleMemory;
use langchain_rust::tools::CommandExecutor;

#[tokio::main]
async fn main() {
    let llm: OpenAIChat<OpenAIConfig> = OpenAIChat::builder()
        .with_model(OpenAIModel::Gpt4oMini)
        .build();
    let memory = SimpleMemory::new();
    let command_executor = CommandExecutor::default().into();
    let agent: Agent<DefaultChainInputCtor> = Agent::builder().tools([command_executor]).build(llm);

    let executor = agent.executor().with_memory(memory.into());

    let input = DefaultChainInput::new("What is the name of the current dir");

    match executor.call(input).await {
        Ok(result) => {
            println!("Result: {result:?}");
        }
        Err(e) => panic!("Error invoking LLMChain: {e:?}"),
    }
}
