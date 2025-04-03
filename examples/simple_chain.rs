use langchain_rust::{
    chain::{Chain, LLMChainBuilder},
    llm::{
        openai::{OpenAI, OpenAIModel},
        OpenAIConfig,
    },
    schemas::MessageType,
    template::MessageTemplate,
    text_replacements,
};
use std::io::{self, Write}; // Include io Library for terminal input

#[tokio::main]
async fn main() {
    let prompt = MessageTemplate::from_jinja2(
        MessageType::HumanMessage,
        "Give me a creative name for a store that sells: {{producto}}",
    );

    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
    let chain = LLMChainBuilder::new()
        .prompt(prompt)
        .llm(llm)
        .build()
        .unwrap();

    print!("Please enter a product: ");
    io::stdout().flush().unwrap(); // Display prompt to terminal

    let mut product = String::new();
    io::stdin().read_line(&mut product).unwrap(); // Get product from terminal input

    let product = product.trim();

    let output = chain
        .invoke(&mut text_replacements! { "producto" => product }.into()) // Use product input here
        .await
        .unwrap();

    println!("Output: {}", output);
}
