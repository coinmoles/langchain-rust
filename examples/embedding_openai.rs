use langchain_rust::embedding::embedder_trait::Embedder;
use langchain_rust::embedding::openai::OpenAiEmbedder;

#[tokio::main]
async fn main() {
    let openai = OpenAiEmbedder::default();

    let response = openai.embed_query("What is the sky blue?").await.unwrap();

    println!("{response:?}");
}
