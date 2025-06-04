use langchain_rust::{
    chain::{Chain, LLMChain},
    llm::{
        openai::{OpenAI, OpenAIModel},
        OpenAIConfig,
    },
    schemas::{ChainInput, ChainInputCtor, MessageType},
    template::MessageTemplate,
};
use std::{
    borrow::Cow,
    io::{self, Write},
}; // Include io Library for terminal input

#[derive(Clone, ChainInput, ChainInputCtor)]
pub struct ProductoInput<'a> {
    #[input(text)]
    pub producto: Cow<'a, str>,
}

#[tokio::main]
async fn main() {
    let prompt = MessageTemplate::from_jinja2(
        MessageType::HumanMessage,
        "Give me a creative name for a store that sells: {{producto}}",
    );

    let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
    let chain: LLMChain<ProductoInputCtor> =
        LLMChain::builder().prompt(prompt).llm(llm).build().unwrap();

    print!("Please enter a product: ");
    io::stdout().flush().unwrap(); // Display prompt to terminal

    let mut product = String::new();
    io::stdin().read_line(&mut product).unwrap(); // Get product from terminal input

    let product = product.trim();

    let output = chain
        .call(&ProductoInput {
            producto: product.into(),
        }) // Use product input here
        .await
        .unwrap();

    println!("Output: {}", output.content);
}
