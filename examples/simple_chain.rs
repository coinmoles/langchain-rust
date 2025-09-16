use std::borrow::Cow;
use std::io::{self, Write};

use async_openai::config::OpenAIConfig;
use langchain_rust::chain::{Chain, ChainInput, Ctor, LLMChain};
use langchain_rust::llm::{OpenAIChat, OpenAIModel};
use langchain_rust::schemas::MessageType;
use langchain_rust::template::MessageTemplate; // Include io Library for terminal input

#[derive(Clone, ChainInput, Ctor)]
pub struct ProductoInput<'a> {
    #[langchain(into = "text")]
    pub producto: Cow<'a, str>,
}

#[tokio::main]
async fn main() {
    let prompt = MessageTemplate::from_jinja2(
        MessageType::Human,
        "Give me a creative name for a store that sells: {{producto}}",
    );

    let llm: OpenAIChat<OpenAIConfig> =
        OpenAIChat::builder().with_model(OpenAIModel::Gpt35).build();
    let chain: LLMChain<ProductoInputCtor> =
        LLMChain::builder().prompt(prompt).llm(llm).build().unwrap();

    print!("Please enter a product: ");
    io::stdout().flush().unwrap(); // Display prompt to terminal

    let mut product = String::new();
    io::stdin().read_line(&mut product).unwrap(); // Get product from terminal input

    let product = product.trim();

    let output = chain
        .call(ProductoInput {
            producto: product.into(),
        }) // Use product input here
        .await
        .unwrap();

    println!("Output: {}", output.content);
}
