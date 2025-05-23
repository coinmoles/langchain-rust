use std::{collections::HashSet, error::Error, pin::Pin};

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    language_models::llm::LLM,
    output_parsers::{OutputParser, SimpleParser},
    schemas::{GenerateResult, GenerateResultContent, InputVariables, Prompt, StreamData},
    template::PromptTemplate,
};

use super::{chain_trait::Chain, ChainError};

pub struct LLMChainBuilder {
    prompt: Option<PromptTemplate>,
    llm: Option<Box<dyn LLM>>,
    output_key: Option<String>,
    output_parser: Option<Box<dyn OutputParser>>,
}

impl LLMChainBuilder {
    pub fn new() -> Self {
        Self {
            prompt: None,
            llm: None,
            output_key: None,
            output_parser: None,
        }
    }

    pub fn prompt<P: Into<PromptTemplate>>(mut self, prompt: P) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn llm<L: Into<Box<dyn LLM>>>(mut self, llm: L) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn output_key<S: Into<String>>(mut self, output_key: S) -> Self {
        self.output_key = Some(output_key.into());
        self
    }

    pub fn output_parser<P: Into<Box<dyn OutputParser>>>(mut self, output_parser: P) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    pub fn build(self) -> Result<LLMChain, ChainError> {
        let prompt = self
            .prompt
            .ok_or_else(|| ChainError::MissingObject("Prompt must be set".into()))?;

        let llm = self
            .llm
            .ok_or_else(|| ChainError::MissingObject("LLM must be set".into()))?;

        let chain = LLMChain {
            prompt,
            llm,
            output_parser: self
                .output_parser
                .unwrap_or_else(|| Box::new(SimpleParser::default())),
        };

        Ok(chain)
    }
}

impl Default for LLMChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LLMChain {
    prompt: PromptTemplate,
    llm: Box<dyn LLM>,
    output_parser: Box<dyn OutputParser>,
}

#[async_trait]
impl Chain for LLMChain {
    fn get_input_keys(&self) -> HashSet<String> {
        self.prompt.variables()
    }

    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let prompt = self.prompt.format(input_variables)?;
        let mut output = self.llm.generate(prompt.to_messages()).await?;

        if let GenerateResultContent::Text(content) = &mut output.content {
            *content = self.output_parser.parse(content).await?;
        }

        log::trace!("\nLLM output:\n{}", output.content);
        if let Some(ref usage) = output.usage {
            log::trace!("\nToken usage:\n{}", usage);
        }

        Ok(output)
    }

    async fn stream(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let prompt = self.prompt.format(input_variables)?;
        let llm_stream = self.llm.stream(prompt.to_messages()).await?;

        // Map the errors from LLMError to ChainError
        let mapped_stream = llm_stream.map_err(ChainError::from);

        Ok(Box::pin(mapped_stream))
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>> {
        let prompt = self.prompt.format(inputs)?;

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use async_openai::config::OpenAIConfig;

    use crate::{
        llm::openai::{OpenAI, OpenAIModel},
        prompt_template,
        schemas::MessageType,
        template::MessageTemplate,
        text_replacements,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_invoke_chain() {
        let mut input_variables: InputVariables = text_replacements! {
            "nombre" => "Juan",
        }
        .into();

        // Create an AI message prompt template
        let human_message_prompt =
            MessageTemplate::from_fstring(MessageType::HumanMessage, "Mi nombre es: {nombre} ");

        // Use the `message_formatter` macro to construct the formatter
        let prompt = prompt_template!(human_message_prompt);

        let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
        let chain = LLMChainBuilder::new()
            .prompt(prompt)
            .llm(llm)
            .build()
            .expect("Failed to build LLMChain");

        // Execute `chain.invoke` and assert that it should succeed
        let result = chain.invoke(&mut input_variables).await;
        assert!(
            result.is_ok(),
            "Error invoking LLMChain: {:?}",
            result.err()
        )
    }
}
