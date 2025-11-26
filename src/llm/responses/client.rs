use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::responses::Response;
use async_trait::async_trait;

use crate::llm::options::LLMOptions;
use crate::llm::responses::helper::{construct_output, generate};
use crate::llm::{LLM, LLMError, OpenAIModel, ResponsesRequest};
use crate::schemas::{IntoWithUsage, LLMOutput, Message, Prompt, Role, ToolSpec, WithUsage};

#[derive(Clone)]
pub struct OpenAIResponses<C: Config> {
    client: OpenAIClient<C>,
    model: String,
    default_options: LLMOptions,
}

impl<C: Config> OpenAIResponses<C> {
    pub fn new<S>(client: OpenAIClient<C>, model: S, options: LLMOptions) -> Self
    where
        S: Into<String>,
    {
        Self {
            client,
            model: model.into(),
            default_options: options,
        }
    }

    fn process_prompt(&self, prompt: Prompt) -> Vec<Message> {
        let mut messages = prompt.to_messages();
        for message in messages.iter_mut() {
            if self.default_options.system_is_assistant.unwrap_or(false)
                && message.role == Role::System
            {
                message.role = Role::Ai;
            }
        }
        messages
    }

    fn build_request(
        &self,
        messages: Vec<Message>,
        tools: Option<&ToolSpec>,
        options: LLMOptions,
    ) -> ResponsesRequest {
        let options = self.default_options.clone().merge(options);
        ResponsesRequest::new(&self.model, messages, tools.cloned()).with_options(options)
    }

    async fn send_request(&self, request: ResponsesRequest) -> Result<Response, LLMError> {
        let stream = self.default_options.stream.unwrap_or(false);
        let response = generate(&self.client, request, stream).await?;
        Ok(response)
    }
}

impl Default for OpenAIResponses<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            OpenAIClient::default(),
            OpenAIModel::Gpt4oMini,
            LLMOptions::default(),
        )
    }
}

#[async_trait]
impl<C: Config + Send + Sync + 'static> LLM for OpenAIResponses<C> {
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
        options: LLMOptions,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        let messages = self.process_prompt(prompt);
        let request = self.build_request(messages, tools, options);
        let response = self.send_request(request).await?;
        let result = construct_output(response.output)?;
        let usage = response.usage.map(Into::into);
        Ok(result.with_usage(usage))
    }

    fn with_default_options(&mut self, options: LLMOptions) {
        self.default_options.merge_inplace(options)
    }
}
