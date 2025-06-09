use base64::prelude::*;
use langchain_rust::{
    chain::{Chain, LLMChain},
    llm::OpenAI,
    prompt_template,
    schemas::{DefaultChainInput, DefaultChainInputCtor, Message, MessageType},
    template::MessageTemplate,
};

#[tokio::main]
async fn main() {
    // Convert image to base64. Can also pass a link to an image instead.
    let image = std::fs::read("./src/llm/test_data/example.jpg").unwrap();
    let image_base64 = BASE64_STANDARD.encode(image);

    let prompt = prompt_template![
        MessageTemplate::from_fstring(MessageType::HumanMessage, "{input}"),
        Message::new_human_message("")
            .with_images(vec![format!("data:image/jpeg;base64,{image_base64}")])
    ];

    // let open_ai = OpenAI::new(langchain_rust::llm::ollama::openai::OllamaConfig::default())
    //     .with_model("llava");
    let open_ai = OpenAI::default();
    let chain: LLMChain<DefaultChainInputCtor> = LLMChain::builder()
        .prompt(prompt)
        .llm(open_ai)
        .build()
        .unwrap();

    match chain
        .call(DefaultChainInput::new("Describe this image"))
        .await
    {
        Ok(result) => {
            println!("Result: {:?}", result.content);
        }
        Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    }
}
