use std::{collections::HashSet, error::Error, pin::Pin};

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    chain::{Chain, ChainError},
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::{GenerateResult, GenerateResultContent, InputVariables, Prompt, StreamData},
    template::PromptTemplate,
};

use super::LLMChainBuilder;

pub struct LLMChain {
    pub(super) prompt: PromptTemplate,
    pub(super) llm: Box<dyn LLM>,
    pub(super) output_parser: Box<dyn OutputParser>,
}

impl LLMChain {
    pub fn builder<'b>() -> LLMChainBuilder<'b> {
        LLMChainBuilder::new()
    }
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
        let chain = LLMChain::builder()
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
