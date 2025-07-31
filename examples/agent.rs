use langchain_rust::{
    agent::{Agent, ConversationalAgent},
    chain::Chain,
    chain::{DefaultChainInput, DefaultChainInputCtor},
    llm::{
        openai::{OpenAI, OpenAIModel},
        OpenAIConfig,
    },
    memory::SimpleMemory,
    tools::CommandExecutor,
};

#[tokio::main]
async fn main() {
    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4Turbo).build();
    let memory = SimpleMemory::new();
    let command_executor = CommandExecutor::default();
    let agent: ConversationalAgent<DefaultChainInputCtor> = ConversationalAgent::builder()
        .tools([command_executor])
        .build(llm);

    let executor = agent.executor().with_memory(memory.into());

    let input = DefaultChainInput::new("What is the name of the current dir");

    match executor.call(input).await {
        Ok(result) => {
            println!("Result: {result:?}");
        }
        Err(e) => panic!("Error invoking LLMChain: {e:?}"),
    }
}
