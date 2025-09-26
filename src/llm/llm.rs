use async_trait::async_trait;

use crate::llm::options::LLMOptions;
use crate::llm::{LLMError, LLMOutput, LLMStream, LlmCapabilities};
use crate::schemas::{Prompt, ToolSpec, WithUsage};

/// A wrapper arround Large Language Models (LLMs).
///
/// This trait defines a common interface for interacting with LLM backends.
/// The methods defined here accepts crate-specific schema types.
/// The implementors should convert these into the format required by the API.
#[async_trait]
pub trait LLM: Sync + Send {
    /// Returns the capabilities of the LLM.
    fn capabilities(&self) -> LlmCapabilities;

    /// Generates a response from the LLM based on the provided prompt.
    async fn generate(
        &self,
        prompt: Prompt,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<LLMOutput>, LLMError>;

    /// Invokes the LLM with a single human message as prompt.
    async fn invoke(&self, msg: &str) -> Result<String, LLMError> {
        let prompt = Prompt::single(msg);
        let result = self.generate(prompt, None).await?.content.to_string();
        Ok(result)
    }

    /// Generates a response from the LLM based on the provided prompt in a stream.
    async fn stream(&self, prompt: Prompt, tools: Option<&ToolSpec>)
    -> Result<LLMStream, LLMError>;

    /// Configure the call options for the LLM.
    ///
    /// This includes parameters like temperature, max tokens, etc.
    fn with_options(&mut self, options: LLMOptions);
}

impl<L> From<L> for Box<dyn LLM>
where
    L: 'static + LLM,
{
    fn from(llm: L) -> Self {
        Box::new(llm)
    }
}
