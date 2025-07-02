#[cfg(feature = "ollama")]
use langchain_rust::llm::LLM;

#[cfg(feature = "ollama")]
#[tokio::main]
async fn main() {
    use langchain_rust::llm::{OpenAI, OpenAIConfig};

    let ollama = OpenAI::builder()
        .with_api_config(OpenAIConfig::default().with_api_base("Ollama API base"))
        .with_model("llama3.2")
        .build();

    let response = ollama.invoke("Hi").await.unwrap();
    println!("{response}");
}

#[cfg(not(feature = "ollama"))]
fn main() {
    println!("This example requires the 'ollama' feature to be enabled.");
    println!("Please run the command as follows:");
    println!("cargo run --example llm_ollama --features=ollama");
}
