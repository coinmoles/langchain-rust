use async_openai::{config::Config, Client as OpenAIClient};
use reqwest::Client;

use crate::llm::options::CallOptions;

use super::OpenAI;

pub struct OpenAIBuilder<C: Config> {
    pub api_config: C,
    pub model: String,
    pub call_options: CallOptions,
    pub http_client: Option<Client>,
}

impl<C: Config + Default> Default for OpenAIBuilder<C> {
    fn default() -> Self {
        OpenAIBuilder {
            api_config: C::default(),
            model: "gpt-3.5-turbo".to_string(),
            call_options: CallOptions::default(),
            http_client: None,
        }
    }
}

impl<C: Config> OpenAIBuilder<C> {
    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.api_config = api_config;
        self
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_call_options(mut self, call_options: CallOptions) -> Self {
        self.call_options = call_options;
        self
    }

    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn build(self) -> OpenAI<C> {
        let client = {
            let mut client = OpenAIClient::with_config(self.api_config);
            if let Some(http_client) = self.http_client {
                client = client.with_http_client(http_client)
            }

            client
        };

        OpenAI::new(client, self.model, self.call_options)
    }
}
