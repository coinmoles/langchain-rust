#![allow(dead_code)]

use crate::embedding::{embedder_trait::Embedder, EmbedderError};
pub use async_openai::config::{AzureConfig, Config, OpenAIConfig};
use async_openai::{
    types::{CreateEmbeddingRequestArgs, EmbeddingInput},
    Client as OpenAIClient,
};
use async_trait::async_trait;

#[derive(Debug)]
pub struct OpenAiEmbedder<C: Config> {
    client: OpenAIClient<C>,
    model: String,
}

impl<C: Config + Send + Sync + 'static> From<OpenAiEmbedder<C>> for Box<dyn Embedder> {
    fn from(val: OpenAiEmbedder<C>) -> Self {
        Box::new(val)
    }
}

impl<C: Config> OpenAiEmbedder<C> {
    pub fn new(config: C) -> Self {
        let client = OpenAIClient::with_config(config);
        OpenAiEmbedder {
            client,
            model: String::from("text-embedding-ada-002"),
        }
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.client = OpenAIClient::with_config(api_config);
        self
    }
}

impl Default for OpenAiEmbedder<OpenAIConfig> {
    fn default() -> Self {
        OpenAiEmbedder::new(OpenAIConfig::default())
    }
}

#[async_trait]
impl<C: Config + Send + Sync> Embedder for OpenAiEmbedder<C> {
    async fn embed_documents(&self, documents: &[String]) -> Result<Vec<Vec<f64>>, EmbedderError> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(EmbeddingInput::StringArray(documents.into()))
            .build()?;

        let response = self.client.embeddings().create(request).await?;

        let embeddings = response
            .data
            .into_iter()
            .map(|item| item.embedding)
            .map(|embedding| {
                embedding
                    .into_iter()
                    .map(|x| x as f64)
                    .collect::<Vec<f64>>()
            })
            .collect();

        Ok(embeddings)
    }

    async fn embed_query(&self, text: &str) -> Result<Vec<f64>, EmbedderError> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(text)
            .build()?;

        let mut response = self.client.embeddings().create(request).await?;

        let item = response.data.swap_remove(0);

        Ok(item
            .embedding
            .into_iter()
            .map(|x| x as f64)
            .collect::<Vec<f64>>())
    }
}
