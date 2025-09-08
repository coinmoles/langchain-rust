use async_openai::{config::Config, Client as OpenAIClient};
use reqwest::Client;

use crate::{
    instructor::{DefaultInstructor, Instructor},
    llm::options::CallOptions,
};

use super::GenericChat;

pub struct GenericChatBuilder<C: Config> {
    pub api_config: C,
    pub model: String,
    pub instructor: Box<dyn Instructor>,
    pub call_options: CallOptions,
    pub http_client: Option<Client>,
}

impl<C: Config + Default> Default for GenericChatBuilder<C> {
    fn default() -> Self {
        GenericChatBuilder {
            api_config: C::default(),
            model: "gpt-3.5-turbo".to_string(),
            instructor: Box::new(DefaultInstructor),
            call_options: CallOptions::default(),
            http_client: None,
        }
    }
}

impl<C: Config> GenericChatBuilder<C> {
    pub fn with_api_config(mut self, api_config: C) -> Self {
        self.api_config = api_config;
        self
    }

    pub fn with_model<S: Into<String>>(mut self, model: S) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
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

    pub fn build(self) -> GenericChat<C> {
        let client = {
            let mut client = OpenAIClient::with_config(self.api_config);
            if let Some(http_client) = self.http_client {
                client = client.with_http_client(http_client)
            }

            client
        };

        GenericChat::new(client, self.model, self.instructor, self.call_options)
    }
}
