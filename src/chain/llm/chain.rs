use std::pin::Pin;

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    chain::{Chain, ChainError},
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::{InputVariableCtor, LLMOutput, OutputVariable, Prompt, StreamData, WithUsage},
    template::PromptTemplate,
};

use super::LLMChainBuilder;

pub struct LLMChain<I, O = String>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    pub(super) prompt: PromptTemplate,
    pub(super) llm: Box<dyn LLM>,
    pub(super) output_parser: Box<dyn OutputParser>,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> LLMChain<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    pub fn builder() -> LLMChainBuilder<I, O> {
        LLMChainBuilder::new()
    }

    pub async fn call_llm(
        &self,
        input_variables: &I::InputVariable<'_>,
    ) -> Result<WithUsage<LLMOutput>, ChainError> {
        let prompt = self.prompt.format(input_variables)?;
        let mut output = self.llm.generate(prompt.to_messages()).await?;

        if let LLMOutput::Text(content) = &mut output.content {
            *content = self.output_parser.parse(content).await?;
        }

        log::trace!("\nLLM output:\n{}", output.content);
        if let Some(ref usage) = output.usage {
            log::trace!("\nToken usage:\n{}", usage);
        }

        Ok(output)
    }
}

#[async_trait]
impl<I, O> Chain for LLMChain<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
{
    type InputCtor = I;
    type Output = O;

    async fn call<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<WithUsage<O>, ChainError>
    where
        'i: 'i2,
    {
        let llm_output = self.call_llm(input).await?;
        let content = llm_output.content.into_text()?;
        let content = O::try_from_string(content)?;

        Ok(WithUsage {
            content,
            usage: llm_output.usage,
        })
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        let prompt = self.prompt.format(input)?;
        let llm_stream = self.llm.stream(prompt.to_messages()).await?;

        // Map the errors from LLMError to ChainError
        let mapped_stream = llm_stream.map_err(ChainError::from);

        Ok(Box::pin(mapped_stream))
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn std::error::Error + Send + Sync>>
    where
        'i: 'i2,
    {
        let prompt = self.prompt.format(input)?;

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_openai::config::OpenAIConfig;

    use crate::{
        llm::openai::{OpenAI, OpenAIModel},
        prompt_template,
        schemas::{InputVariable, MessageType, TextReplacements},
        template::MessageTemplate,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_invoke_chain() {
        pub struct NombreInputCtor;
        impl InputVariableCtor for NombreInputCtor {
            type InputVariable<'a> = NombreInput<'a>;
        }

        pub struct NombreInput<'a> {
            pub nombre: &'a str,
        }
        impl InputVariable for NombreInput<'_> {
            fn text_replacements(&self) -> TextReplacements {
                HashMap::from([("nombre", self.nombre.into())])
            }
        }

        let input = NombreInput { nombre: "Juan" };

        // Create an AI message prompt template
        let human_message_prompt =
            MessageTemplate::from_fstring(MessageType::HumanMessage, "Mi nombre es: {nombre} ");

        // Use the `message_formatter` macro to construct the formatter
        let prompt = prompt_template!(human_message_prompt);

        let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
        let chain: LLMChain<NombreInputCtor, String> = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .build()
            .expect("Failed to build LLMChain");

        // Execute `chain.invoke` and assert that it should succeed
        let result = chain.call(&input).await;
        assert!(
            result.is_ok(),
            "Error invoking LLMChain: {:?}",
            result.err()
        )
    }
}
