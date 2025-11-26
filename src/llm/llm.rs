use async_trait::async_trait;

use crate::agent::AgentError;
use crate::llm::options::LLMOptions;
use crate::llm::{DefaultSession, LLMError, LlmSession};
use crate::schemas::{LLMOutput, Message, Prompt, ToolSpec, WithUsage};

/// A common interface for interacting with LLM backends.
///
/// The methods in this trait accepts types defined in this crate. The implementors should convert
/// these into the format required by the API.
#[async_trait]
pub trait LLM: Sync + Send {
    /// Generates a response from the LLM with the provided prompt.
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
        options: LLMOptions,
    ) -> Result<WithUsage<LLMOutput>, LLMError>;

    /// Generates a response from the LLM with the provided prompt using the default option.
    async fn generate_default(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError> {
        self.generate(prompt, tools, LLMOptions::default()).await
    }

    /// Generates a response from the LLM with a single human message.
    async fn invoke(&self, msg: &str) -> Result<String, LLMError> {
        let prompt = Prompt::single(msg);
        let result = self
            .generate_default(prompt, None)
            .await?
            .content
            .to_string();
        Ok(result)
    }

    /// Begins a new session with the agent LLM.
    ///
    /// The types defined in this crate may not be sufficient to represent a prolonged interaction
    /// with an agentic LLM, as some providers (like Claude) use a unique api.
    ///
    /// By default, [`DefaultSession`] is used, which does not preserve any model-specific fields.
    /// Implement a model-specific [`LlmSession`] to store the interaction in model-specific types
    /// to do so.
    async fn begin_session<'a>(
        &'a self,
        prompt: Vec<Message>,
        tools: Option<ToolSpec>,
    ) -> Result<Box<dyn LlmSession + 'a>, AgentError> {
        Ok(Box::new(DefaultSession::new(self, prompt, tools)))
    }

    /// Configures the call options to be used in subsequent requests.
    fn with_default_options(&mut self, options: LLMOptions);
}

impl<L> From<L> for Box<dyn LLM>
where
    L: 'static + LLM,
{
    fn from(llm: L) -> Self {
        Box::new(llm)
    }
}
