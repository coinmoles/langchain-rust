use async_openai::Client as OpenAIClient;
use async_openai::config::Config;
use reqwest::Client;

use super::GenericChat;
use crate::llm::options::CallOptions;
use crate::llm::{DefaultInstructor, Instructor, OpenAIModel};

/// A builder for constructing a [`GenericChat`].
pub struct GenericChatBuilder<C: Config> {
    /// The HTTP client to use.
    http_client: Option<Client>,
    /// The API configuration.
    api_config: C,
    /// The model id.
    model: String,
    /// The [`Instructor`] used to create tool use instruction and parse tool calls.
    instructor: Box<dyn Instructor>,
    /// The call options.
    call_options: CallOptions,
}

impl<C: Config + Default> GenericChatBuilder<C> {
    /// Creates a new [`GenericChatBuilder`].
    ///
    /// This is the same as calling [`GenericChat::builder`].
    #[must_use]
    pub fn new() -> Self {
        GenericChatBuilder {
            api_config: C::default(),
            model: OpenAIModel::Gpt4oMini.to_string(),
            instructor: Box::new(DefaultInstructor),
            call_options: CallOptions::default(),
            http_client: None,
        }
    }
}

impl<C: Config> GenericChatBuilder<C> {
    /// Sets the HTTP client to use.
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
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Sets the instructor.
    ///
    /// Instructor is used to used to create tool use instruction and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
        self
    }

    /// Sets the call options.
    pub fn with_call_options(mut self, call_options: CallOptions) -> Self {
        self.call_options = call_options;
        self
    }

    /// Returns a [`GenericChat`] that uses this [`GenericChatBuilder`] configuration.
    pub fn build(self) -> GenericChat<C> {
        let http_client = self.http_client.unwrap_or_default();
        let client = OpenAIClient::build(http_client, self.api_config, Default::default());
        GenericChat::new(client, self.model, self.instructor, self.call_options)
    }
}

impl<C: Config + Default> Default for GenericChatBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}
