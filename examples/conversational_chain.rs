use std::io::{stdout, Write};

use futures_util::StreamExt;
use langchain_rust::{
    chain::{Chain, ConversationalChain},
    llm::{openai::OpenAI, OpenAIConfig, OpenAIModel},
    memory::SimpleMemory,
    schemas::{DefaultChainInput, DefaultChainInputCtor},
};

#[tokio::main]
async fn main() {
    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
    //We initialise a simple memory. By default conversational chain have this memory, but we
    //initialise it as an example, if you dont want to have memory use DummyMemory
    let memory = SimpleMemory::new();

    let chain: ConversationalChain<DefaultChainInputCtor> = ConversationalChain::builder()
        .llm(llm)
        //IF YOU WANT TO ADD A CUSTOM PROMPT YOU CAN UN COMMENT THIS:
        //         .prompt(message_formatter![
        //             fmt_message!(Message::new_system_message("You are a helpful assistant")),
        //             fmt_template!(HumanMessagePromptTemplate::new(
        //             template_fstring!("
        // The following is a friendly conversation between a human and an AI. The AI is talkative and provides lots of specific details from its context. If the AI does not know the answer to a question, it truthfully says it does not know.
        //
        // Current conversation:
        // {history}
        // Human: {input}
        // AI:
        // ",
        //             "input","history")))
        //
        //         ])
        .memory(memory.into())
        .build()
        .expect("Error building ConversationalChain");

    let input = DefaultChainInput::new("I'm from Peru");

    let mut stream = chain.stream(&input).await.unwrap();
    while let Some(result) = stream.next().await {
        match result {
            Ok(data) => {
                //If you just want to print to stdout, you can use data.to_stdout().unwrap();
                print!("{}", data.content);
                stdout().flush().unwrap();
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    let input = DefaultChainInput::new("Which are the typical dish");
    match chain.call(&input).await {
        Ok(result) => {
            println!("\n");
            println!("Result: {}", result.content);
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
