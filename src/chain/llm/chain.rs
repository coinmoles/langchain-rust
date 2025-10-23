use std::borrow::Borrow;

use async_trait::async_trait;

use super::LLMChainBuilder;
use crate::chain::{Chain, ChainError, ChainOutput, GetPrompt, InputCtor, OutputCtor, StringCtor};
use crate::llm::LLM;
use crate::output_parser::OutputParser;
use crate::schemas::{IntoWithUsage, LLMEvent, LLMStream, Prompt, ToolSpec, WithUsage};
use crate::template::{PromptTemplate, TemplateError};

pub struct LLMChain<I: InputCtor, O: OutputCtor = StringCtor>
where
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) prompt: PromptTemplate,
    pub(super) llm: Box<dyn LLM>,
    pub(super) output_parser: Box<dyn OutputParser<I, O>>,
    pub(super) _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I: InputCtor, O: OutputCtor> LLMChain<I, O>
where
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn builder() -> LLMChainBuilder<I, O> {
        LLMChainBuilder::new()
    }

    pub async fn call_llm(
        &self,
        input: &I::Target<'_>,
        tools: Option<&ToolSpec>,
    ) -> Result<WithUsage<O::Target<'static>>, ChainError> {
        let prompt = self.prompt.format(input)?;
        let WithUsage { content, usage } = self.llm.generate(prompt, tools).await?;

        log::trace!("\nLLM output:\n{content}");
        if let Some(usage) = &usage {
            log::trace!("\nToken usage:\n{usage}");
        }

        let content = match content.event {
            LLMEvent::Text(text) => self.output_parser.parse_from_text(text),
            LLMEvent::ToolCall(tool_calls) => {
                O::Target::from_tool_call(content.thought, tool_calls)
            }
        }?;

        Ok(content.with_usage(usage))
    }

    pub async fn stream_llm(
        &self,
        input: &I::Target<'_>,
        tools: Option<&ToolSpec>,
    ) -> Result<LLMStream, ChainError> {
        let prompt = self.prompt.format(input.borrow())?;
        let stream = self.llm.stream(prompt, tools).await?;
        Ok(stream)
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> Chain<I, O> for LLMChain<I, O>
where
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let prompt = self.prompt.format(&input)?;
        let WithUsage { content, usage } = self.llm.generate(prompt, None).await?;

        log::trace!("\nLLM output:\n{content}");
        if let Some(usage) = &usage {
            log::trace!("\nToken usage:\n{usage}");
        }

        let content = match content.event {
            LLMEvent::Text(text) => self.output_parser.parse_from_text_and_input(input, text)?,
            LLMEvent::ToolCall(tool_calls) => {
                O::Target::from_tool_call(content.thought, tool_calls)?
            }
        };

        Ok(content.with_usage(usage))
    }

    async fn stream(&self, input: I::Target<'_>) -> Result<LLMStream, ChainError> {
        self.stream_llm(&input, None).await
    }
}

impl<I: InputCtor, O: OutputCtor> GetPrompt<&I::Target<'_>> for LLMChain<I, O>
where
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

    use super::*;
    use crate::chain::{Chain, ChainInput, Ctor};
    use crate::llm::{GenericChat, OpenAIModel};
    use crate::prompt_template;
    use crate::schemas::Role;
    use crate::template::MessageTemplate;

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
            MessageTemplate::from_fstring(Role::Human, "Mi nombre es: {nombre} ");

        // Use the `message_formatter` macro to construct the formatter
        let prompt = prompt_template!(human_message_prompt);

        let llm: GenericChat<OpenAIConfig> = GenericChat::builder()
            .with_model(OpenAIModel::Gpt35)
            .build();
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
