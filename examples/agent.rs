use langchain_rust::{
    agent::{AgentExecutor, ConversationalAgentBuilder},
    chain::Chain,
    llm::{
        openai::{OpenAI, OpenAIModel},
        OpenAIConfig,
    },
    memory::SimpleMemory,
    schemas::InputVariables,
    text_replacements,
    tools::{CommandExecutor, ToolFunction},
};

#[tokio::main]
async fn main() {
    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt4Turbo).build();
    let memory = SimpleMemory::new();
    let command_executor = CommandExecutor::default();
    let agent = ConversationalAgentBuilder::new()
        .tools(vec![command_executor.into_boxed_tool()])
        .build(llm)
        .await
        .unwrap();

    let executor = AgentExecutor::from_agent(agent).with_memory(memory.into());

    let mut input_variables: InputVariables = text_replacements! {
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
