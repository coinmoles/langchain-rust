use {std::borrow::Borrow, std::pin::Pin};

use async_trait::async_trait;
use futures::{Stream, TryStreamExt};

use crate::{
    chain::{Chain, ChainError},
    language_models::llm::LLM,
    output_parser::OutputParser,
    schemas::{
        ChainOutput, GetPrompt, InputCtor, IntoWithUsage, LLMOutput, OutputCtor, Prompt,
        StreamData, StringCtor, WithUsage,
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
    pub(super) output_parser: Box<dyn OutputParser<I, O>>,
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

    pub(crate) async fn call_with_reference(
        &self,
        input: &I::Target<'_>,
    ) -> Result<WithUsage<O::Target<'static>>, ChainError> {
        let prompt = self.prompt.format(input)?;
        let WithUsage { content, usage } = self.llm.generate(prompt.to_messages()).await?;

        log::trace!("\nLLM output:\n{content}");
        if let Some(usage) = &usage {
            log::trace!("\nToken usage:\n{usage}");
        }

        let content = match content {
            LLMOutput::Text(text) => self.output_parser.parse_from_text(text),
            LLMOutput::ToolCall(tool_calls) => O::Target::construct_from_tool_call(tool_calls),
        }?;

        Ok(content.with_usage(usage))
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
        let prompt = self.prompt.format(&input)?;
        let WithUsage { content, usage } = self.llm.generate(prompt.to_messages()).await?;

        log::trace!("\nLLM output:\n{content}");
        if let Some(usage) = &usage {
            log::trace!("\nToken usage:\n{usage}");
        }

        let content = match content {
            LLMOutput::Text(text) => self.output_parser.parse_from_text_and_input(input, text),
            LLMOutput::ToolCall(tool_calls) => O::Target::construct_from_tool_call(tool_calls),
        }?;

        Ok(content.with_usage(usage))
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
