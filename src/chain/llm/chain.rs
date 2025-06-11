use {std::borrow::Borrow, std::pin::Pin};

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    chain::{Chain, ChainError},
    language_models::llm::LLM,
    output_parsers::OutputParser,
    schemas::{
        ChainOutput, GetPrompt, InputCtor, LLMOutput, OutputCtor, Prompt, StreamData, StringCtor,
        WithUsage,
    },
    template::{PromptTemplate, TemplateError},
};

use super::LLMChainBuilder;

pub struct LLMChain<I, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) prompt: PromptTemplate,
    pub(super) llm: Box<dyn LLM>,
    pub(super) output_parser: Box<dyn OutputParser>,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> LLMChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
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

    pub async fn stream_llm(
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
impl<I, O> Chain for LLMChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let llm_output = self.call_llm(&input).await?;
        let content = llm_output.content.into_text()?;
        let content = O::Target::parse_output(input, content)?;

        Ok(WithUsage {
            content,
            usage: llm_output.usage,
        })
    }

    async fn stream(
        &self,
        input: I::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.stream_llm(&input).await
    }
}

impl<I, O> GetPrompt<&I::Target<'_>> for LLMChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: &I::Target<'_>) -> Result<Prompt, TemplateError> {
        let prompt = self.prompt.format(input)?;
        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use async_openai::config::OpenAIConfig;

    use crate::{
        chain::Chain,
        llm::openai::{OpenAI, OpenAIModel},
        prompt_template,
        schemas::{ChainInput, Ctor, MessageType},
        template::MessageTemplate,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_invoke_chain() {
        #[derive(Clone, ChainInput, Ctor)]
        #[allow(dead_code)]
        pub struct NombreInput<'a> {
            #[langchain(into = "text")]
            pub nombre: &'a str,
        }

        let input = NombreInput { nombre: "Juan" };

        // Create an AI message prompt template
        let human_message_prompt =
            MessageTemplate::from_fstring(MessageType::Human, "Mi nombre es: {nombre} ");

        // Use the `message_formatter` macro to construct the formatter
        let prompt = prompt_template!(human_message_prompt);

        let llm: OpenAI<OpenAIConfig> = OpenAI::builder().with_model(OpenAIModel::Gpt35).build();
        let chain: LLMChain<NombreInputCtor> = LLMChain::builder()
            .prompt(prompt)
            .llm(llm)
            .build()
            .expect("Failed to build LLMChain");

        // Execute `chain.invoke` and assert that it should succeed
        let result = chain.call(input).await;
        assert!(
            result.is_ok(),
            "Error invoking LLMChain: {:?}",
            result.err()
        )
    }
}
