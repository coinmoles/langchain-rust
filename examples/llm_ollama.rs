use async_openai::config::OpenAIConfig;
use langchain_rust::llm::{LLM, OpenAIChat};

#[tokio::main]
async fn main() {
    let ollama = OpenAIChat::builder()
        .with_api_config(OpenAIConfig::default().with_api_base("Ollama API base"))
        .with_model("llama3.2")
        .build();

    let response = ollama.invoke("Hi").await.unwrap();
    println!("{response}");
}
