use indoc::indoc;
use langchain_rust::{
    chain::{Chain, StuffDocument, StuffQABuilder},
    llm::openai::OpenAI,
    schemas::Document,
};

#[tokio::main]
async fn main() {
    let llm = OpenAI::default();

    let chain = StuffDocument::builder()
        .llm(llm)
        // .prompt() you can add a custom prompt if you want
        .build()
        .unwrap();
    let mut input = StuffQABuilder::new()
        .question("How old is luis and whats his favorite text editor")
        .documents(&[
            Document::new(indoc! {"
                Question: Which is the favorite text editor of luis
                Answer: Nvim"
            }),
            Document::new(indoc! {"
                Question: How old is Luis
                Answer: 24"
            }),
        ])
        .build()
        .into();

    let output = chain.invoke(&mut input).await.unwrap();

    println!("{}", output);
}
