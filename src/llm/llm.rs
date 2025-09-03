use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::{
    llm::{options::CallOptions, LLMError, LLMOutput},
    schemas::{Message, StreamData, WithUsage},
};

#[async_trait]
pub trait LLM: Sync + Send {
    async fn generate(&self, messages: Vec<Message>) -> Result<WithUsage<LLMOutput>, LLMError>;

    async fn invoke(&self, prompt: &str) -> Result<String, LLMError> {
        let result = self
            .generate(vec![Message::new_human_message(prompt)])
            .await?
            .content
            .into_text()?;
        Ok(result)
    }

    async fn stream(
        &self,
        _messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError>;

    /// This is useful when you want to create a chain and override
    /// LLM options
    fn add_call_options(&mut self, call_options: CallOptions);

    //This is usefull when using non chat models
    fn messages_to_string(&self, messages: &[Message]) -> String {
        messages
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl<L> From<L> for Box<dyn LLM>
where
    L: 'static + LLM,
{
    fn from(llm: L) -> Self {
        Box::new(llm)
    }
}
