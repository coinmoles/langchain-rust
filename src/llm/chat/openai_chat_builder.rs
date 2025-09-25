use async_openai::Client as OpenAIClient;
use async_openai::config::Config;
use reqwest::Client;

use super::OpenAIChat;
use crate::llm::options::CallOptions;

/// A builder for constructing a [`OpenAIChat`].
pub struct OpenAIChatBuilder<C: Config> {
    /// The HTTP client to use.
    http_client: Option<Client>,
    /// The API configuration.
    api_config: C,
    /// The model id.
    model: String,
    /// The call options.
    call_options: CallOptions,
}

impl<C: Config + Default> OpenAIChatBuilder<C> {
    /// Constructs a new [`OpenAIChatBuilder`].
    ///
    /// This is the same as calling [`OpenAIChat::builder`].
    #[must_use]
    pub fn new() -> Self {
        OpenAIChatBuilder {
            api_config: C::default(),
            model: "gpt-3.5-turbo".to_string(),
            call_options: CallOptions::default(),
            http_client: None,
        }
    }
}

impl<C: Config> OpenAIChatBuilder<C> {
    /// Sets the HTTP client to use.
    #[must_use]
    pub fn with_http_client(mut self, http_client: Client) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Sets the API configuration.
    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.api_config = api_config;
        self
    }

    /// Sets the model id.
    ///
    /// You can use the predefined models (e.g.,
    /// [`OpenAIModel::Gpt4oMini`](crate::llm::OpenAIModel::Gpt4oMini)) or provide a custom model
    /// name with a raw string.
    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the call options.
    pub fn with_call_options(mut self, call_options: CallOptions) -> Self {
        self.call_options = call_options;
        self
    }

    /// Returns a [`OpenAIChat`] that uses this [`OpenAIChatBuilder`] configuration.
    pub fn build(self) -> OpenAIChat<C> {
        let client = {
            let mut client = OpenAIClient::with_config(self.api_config);
            if let Some(http_client) = self.http_client {
                client = client.with_http_client(http_client)
            }

            client
        };

        OpenAIChat::new(client, self.model, self.call_options)
    }
}

impl<C: Config + Default> Default for OpenAIChatBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}
