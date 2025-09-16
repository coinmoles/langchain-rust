use async_openai::config::OpenAIConfig;
use langchain_rust::llm::{LLM, OpenAIChat};

#[tokio::main]
async fn main() {
    //OpenAI Example
    let open_ai = OpenAIChat::default();
    let response = open_ai.invoke("hola").await.unwrap();
    println!("{response}");

    //or we can set config as
    let open_ai = OpenAIChat::builder()
        .with_api_config(
            OpenAIConfig::default()
                .with_api_base("xxx") //if you want to specify base url
                .with_api_key("<you_api_key>"), //if you want to set you open ai key,
        )
        .build();

    let response = open_ai.invoke("hola").await.unwrap();
    println!("{response}");
}
