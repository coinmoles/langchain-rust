use crate::{
    language_models::{llm::LLM, GenerateResult, LLMError, TokenUsage},
    schemas::{Message, MessageType, StreamData},
};
use async_trait::async_trait;
use futures::Stream;
use ollama_rs::generation::{images::Image, tools::ToolCall};
pub use ollama_rs::{
    error::OllamaError,
    generation::{
        chat::{request::ChatMessageRequest, ChatMessage, MessageRole},
        options::GenerationOptions,
    },
    Ollama as OllamaClient,
};
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct Ollama {
    pub(crate) client: Arc<OllamaClient>,
    pub(crate) model: String,
    pub(crate) options: Option<GenerationOptions>,
}

/// [llama3.2](https://ollama.com/library/llama3.2) is a 3B parameters, 2.0GB model.
const DEFAULT_MODEL: &str = "llama3.2";

impl Ollama {
    pub fn new<S: Into<String>>(
        client: Arc<OllamaClient>,
        model: S,
        options: Option<GenerationOptions>,
    ) -> Self {
        Ollama {
            client,
            model: model.into(),
            options,
        }
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_options(mut self, options: GenerationOptions) -> Self {
        self.options = Some(options);
        self
    }

    fn generate_request(&self, messages: &[Message]) -> ChatMessageRequest {
        let mapped_messages = messages.iter().map(|message| message.into()).collect();
        ChatMessageRequest::new(self.model.clone(), mapped_messages)
    }
}

impl From<&Message> for ChatMessage {
    fn from(message: &Message) -> Self {
        let images = match message.images.clone() {
            Some(images) => {
                let images = images
                    .iter()
                    .map(|image| Image::from_base64(&image.image_url))
                    .collect();
                Some(images)
            }
            None => None,
        };
        ChatMessage {
            content: message.content.clone(),
            images,
            tool_calls: message
                .tool_calls
                .as_ref()
                .and_then(|tool_calls| {
                    tool_calls
                        .clone()
                        .into_iter()
                        .map(|tc| {
                            serde_json::from_value::<ToolCall>(serde_json::to_value(tc.clone())?)
                        }) // TODO: FIX THIS
                        .collect::<Result<_, _>>()
                        .ok()
                })
                .unwrap_or_default(),
            role: message.message_type.clone().into(),
        }
    }
}

impl From<MessageType> for MessageRole {
    fn from(message_type: MessageType) -> Self {
        match message_type {
            MessageType::AIMessage => MessageRole::Assistant,
            MessageType::ToolMessage => MessageRole::Assistant,
            MessageType::SystemMessage => MessageRole::System,
            MessageType::HumanMessage => MessageRole::User,
        }
    }
}

impl Default for Ollama {
    fn default() -> Self {
        let client = Arc::new(OllamaClient::default());
        Ollama::new(client, String::from(DEFAULT_MODEL), None)
    }
}

#[async_trait]
impl LLM for Ollama {
    async fn generate(&self, messages: &[Message]) -> Result<GenerateResult, LLMError> {
        let request = self.generate_request(messages);
        let result = self.client.send_chat_messages(request).await?;

        let generation = result.message.content;

        let tokens = result.final_data.map(|final_data| {
            let prompt_tokens = final_data.prompt_eval_count as u32;
            let completion_tokens = final_data.eval_count as u32;
            TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            }
        });

        Ok(GenerateResult { tokens, generation })
    }

    async fn stream(
        &self,
        messages: &[Message],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, LLMError>> + Send>>, LLMError> {
        let request = self.generate_request(messages);
        let result = self.client.send_chat_messages_stream(request).await?;

        let stream = result.map(|data| {
            data.map(|data| {
                StreamData::new(
                    serde_json::to_value(&data).unwrap_or_default(),
                    None,
                    data.message.content,
                )
            })
            .map_err(|_| OllamaError::Other("Stream error".to_string()).into())
        });

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio_stream::StreamExt;

    #[tokio::test]
    #[ignore]
    async fn test_generate() {
        let ollama = Ollama::default().with_model("llama3.2");
        let response = ollama.invoke("Hey Macarena, ay").await.unwrap();
        println!("{}", response);
    }

    #[tokio::test]
    #[ignore]
    async fn test_stream() {
        let ollama = Ollama::default().with_model("llama3.2");

        let message = Message::new(
            MessageType::HumanMessage,
            "Why does water boil at 100 degrees?",
        );
        let mut stream = ollama.stream(&[message]).await.unwrap();
        let mut stdout = tokio::io::stdout();
        while let Some(res) = stream.next().await {
            let data = res.unwrap();
            stdout.write_all(data.content.as_bytes()).await.unwrap();
        }
        stdout.write_all(b"\n").await.unwrap();
        stdout.flush().await.unwrap();
    }
}
