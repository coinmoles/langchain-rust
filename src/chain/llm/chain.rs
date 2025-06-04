use std::borrow::Cow;

use {std::borrow::Borrow, std::pin::Pin};

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    chain::{ChainError, ChainImpl},
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::{ChainInputCtor, LLMOutput, ChainOutput, Prompt, StreamData, WithUsage},
    template::PromptTemplate,
};

use super::LLMChainBuilder;

pub struct LLMChain<I, O = String>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    pub(super) prompt: PromptTemplate,
    pub(super) llm: Box<dyn LLM>,
    pub(super) output_parser: Box<dyn OutputParser>,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> LLMChain<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    pub fn builder() -> LLMChainBuilder<I, O> {
        LLMChainBuilder::new()
    }

    pub async fn call_llm(
        &self,
        input: &I::Target<'_>,
    ) -> Result<WithUsage<LLMOutput>, ChainError> {
        let prompt = self.prompt.format(input.borrow())?;
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

    async fn stream_llm(
        &self,
        input: &I::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let prompt = self.prompt.format(input.borrow())?;
        let llm_stream = self.llm.stream(prompt.to_messages()).await?;

        // Map the errors from LLMError to ChainError
        let mapped_stream = llm_stream.map_err(ChainError::from);

        Ok(Box::pin(mapped_stream))
    }
}

#[async_trait]
impl<I, O> ChainImpl for LLMChain<I, O>
where
    I: ChainInputCtor,
    O: ChainOutput,
{
    type InputCtor = I;
    type Output = O;

    async fn call_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<WithUsage<O>, ChainError> {
        let llm_output = self.call_llm(input.as_ref()).await?;
        let content = llm_output.content.into_text()?;
        let content = O::try_from_string(content)?;

        Ok(WithUsage {
            content,
            usage: llm_output.usage,
        })
    }

    async fn stream_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.stream_llm(input.as_ref()).await
    }

    fn get_prompt_impl<'i>(&self, input: Cow<'i, I::Target<'i>>) -> Result<Prompt, ChainError> {
        let prompt = self.prompt.format(input.as_ref())?;

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
        schemas::{ChainInput, MessageType, TextReplacements},
        template::MessageTemplate,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_invoke_chain() {
        pub struct NombreInputCtor;
        impl ChainInputCtor for NombreInputCtor {
            type Target<'a> = NombreInput<'a>;
        }

        #[derive(Clone)]
        pub struct NombreInput<'a> {
            pub nombre: &'a str,
        }
        impl ChainInput for NombreInput<'_> {
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
        let result = chain.call_impl(Cow::Owned(input)).await;
        assert!(
            result.is_ok(),
            "Error invoking LLMChain: {:?}",
            result.err()
        )
    }
}
