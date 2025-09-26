use async_openai::Client as OpenAIClient;
use async_openai::config::{Config, OpenAIConfig};
use async_openai::types::responses::ResponseEvent;
use async_trait::async_trait;

use crate::llm::options::LLMOptions;
use crate::llm::responses::helper::{construct_output, generate, map_stream};
use crate::llm::{
    LLM, LLMError, LLMOutput, LLMStream, LlmCapabilities, OpenAIModel, ResponsesRequest,
};
use crate::schemas::{IntoWithUsage, Message, Prompt, Role, ToolSpec, WithUsage};

#[derive(Clone)]
pub struct OpenAIResponses<C: Config> {
    client: OpenAIClient<C>,
    model: String,
    options: LLMOptions,
}

impl<C: Config> OpenAIResponses<C> {
    pub fn new<S>(client: OpenAIClient<C>, model: S, options: LLMOptions) -> Self
    where
        S: Into<String>,
    {
        Self {
            client,
            model: model.into(),
            options,
        }
    }

    fn process_prompt(&self, prompt: Prompt) -> Vec<Message> {
        let mut messages = prompt.to_messages();
        for message in messages.iter_mut() {
            if self.options.system_is_assistant.unwrap_or(false) && message.role == Role::System {
                message.role = Role::Ai;
            }
        }
        messages
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
    fn capabilities(&self) -> LlmCapabilities {
        LlmCapabilities { native_mcp: false }
    }

    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        let messages = self.process_prompt(prompt);
        let options = self.options.clone();
        let stream = self.options.stream.unwrap_or(false);
        let request =
            ResponsesRequest::new(&self.model, messages, tools.cloned())?.with_options(options);
        let response = generate(&self.client, request, stream).await?;
        let result = construct_output(response.output)?;
        let usage = response.usage.map(Into::into);
        Ok(result.with_usage(usage))
    }

    async fn stream(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<LLMStream, LLMError> {
        let messages = self.process_prompt(prompt);
        let options = self.options.clone();
        let request =
            ResponsesRequest::new(&self.model, messages, tools.cloned())?.with_options(options);

        let stream = self
            .client
            .responses()
            .create_stream_byot::<_, ResponseEvent>(request)
            .await?;
        let new_stream = map_stream(stream);
        Ok(new_stream)
    }

    fn with_options(&mut self, options: LLMOptions) {
        self.options.merge_options(options)
    }
}
