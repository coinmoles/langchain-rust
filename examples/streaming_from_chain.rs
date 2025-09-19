use futures::StreamExt;
use langchain_rust::chain::{Chain, DefaultChainInput, DefaultChainInputCtor, LLMChain};
use langchain_rust::llm::OpenAIChat;
use langchain_rust::prompt_template;
use langchain_rust::schemas::{Message, Role};
use langchain_rust::template::MessageTemplate;

#[tokio::main]
async fn main() {
    let open_ai = OpenAIChat::default();

    let prompt = prompt_template![
        Message::new_system_message("You are world class technical documentation writer."),
        MessageTemplate::from_fstring(Role::Human, "{input}")
    ];

    let chain: LLMChain<DefaultChainInputCtor> = LLMChain::builder()
        .prompt(prompt)
        .llm(open_ai.clone())
        .build()
        .unwrap();

    let mut stream = chain
        .stream(DefaultChainInput::new(
            "Who is the writer of 20,000 Leagues Under the Sea?",
        ))
        .await
        .unwrap();

    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => value.to_stdout().unwrap(),
            Err(e) => panic!("Error invoking LLMChain: {e:?}"),
        }
    }
}
