use langchain_rust::llm::OpenAIConfig;

use langchain_rust::{llm::openai::OpenAI, llm::LLM};

#[tokio::main]
async fn main() {
    //OpenAI Example
    let open_ai = OpenAI::default();
    let response = open_ai.invoke("hola").await.unwrap();
    println!("{response}");

    //or we can set config as
    let open_ai = OpenAI::builder()
        .with_api_config(
            OpenAIConfig::default()
                .with_api_base("xxx") //if you want to specify base url
                .with_api_key("<you_api_key>"), //if you want to set you open ai key,
        )
        .build();

    let response = open_ai.invoke("hola").await.unwrap();
    println!("{response}");
}
