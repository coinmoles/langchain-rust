use langchain_rust::{
    agent::{Agent, ConversationalAgent},
    chain::Chain,
    llm::{
        openai::{OpenAI, OpenAIModel},
        OpenAIConfig,
    },
    memory::SimpleMemory,
    text_replacements,
    tools::CommandExecutor,
};

#[tokio::main]
async fn main() {
    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4Turbo).build();
    let memory = SimpleMemory::new();
    let command_executor = CommandExecutor::default();
    let agent = ConversationalAgent::builder()
        .tools([command_executor])
        .build(llm)
        .await
        .unwrap();

    let executor = agent.executor().with_memory(memory.into());

    let mut input_variables = text_replacements! {
        "input" => "What is the name of the current dir",
    }
    .into();

    match executor.invoke(&mut input_variables).await {
        Ok(result) => {
            println!("Result: {:?}", result);
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
