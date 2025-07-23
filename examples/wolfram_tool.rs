use langchain_rust::tools::{DefaultToolInput, Tool, Wolfram};

#[tokio::main]
async fn main() {
    let wolfram = Wolfram::default().with_excludes(&["Plot"]);
    let input = DefaultToolInput::new("Solve x^2 - 2x + 1 = 0");
    let result = wolfram.run(input).await;

    println!("{:?}", result.unwrap());
}
