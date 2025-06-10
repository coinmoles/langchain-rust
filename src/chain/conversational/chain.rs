use std::{fmt::Display, pin::Pin, sync::Arc};

use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use futures_util::{pin_mut, StreamExt};
use tokio::sync::{Mutex, RwLock};

use crate::{
    chain::{Chain, ChainError, LLMChain},
    language_models::LLMError,
    memory::Memory,
    schemas::{
        messages::Message, ChainOutput, DefaultChainInputCtor, GetPrompt, InputCtor, IntoWithUsage,
        LLMOutput, OutputCtor, Prompt, StreamData, StringCtor, WithUsage,
    },
    template::TemplateError,
};

use super::{ConversationalChainBuilder, ConversationalChainInput, ConversationalChainInputCtor};

pub struct ConversationalChain<I = DefaultChainInputCtor, O = StringCtor>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub(super) llm_chain: LLMChain<ConversationalChainInputCtor<I>>,
    pub memory: Arc<RwLock<dyn Memory>>,
    pub(super) _phantom: std::marker::PhantomData<O>,
}

//Conversational Chain is a simple chain to interact with ai as a string of messages
impl<I, O> ConversationalChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    pub fn builder() -> ConversationalChainBuilder<I, O> {
        ConversationalChainBuilder::new()
    }
}

#[async_trait]
impl<I, O> Chain for ConversationalChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    type InputCtor = I;
    type OutputCtor = O;

    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let human_message = Message::new_human_message(input.to_string());

        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };
        let input = ConversationalChainInput::new(input).with_history(history);
        let result = self.llm_chain.call_llm(&input).await?;

        let mut memory = self.memory.write().await;
        memory.add_message(human_message);

        match &result.content {
            LLMOutput::Text(text) => memory.add_ai_message(text.clone()),
            LLMOutput::ToolCall(tool_calls) => memory.add_tool_call_message(tool_calls.clone()),
            LLMOutput::Refusal(refusal) => return Err(LLMError::OtherError(refusal.into()).into()),
        }

        Ok(
            O::Target::parse_output(input.inner, result.content.into_text()?)?
                .with_usage(result.usage),
        )
    }

    async fn stream(
        &self,
        input: I::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let human_message = Message::new_human_message(input.to_string());

        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };
        let input = ConversationalChainInput::new(input).with_history(history);

        let complete_ai_message = Arc::new(Mutex::new(String::new()));
        let complete_ai_message_clone = complete_ai_message.clone();

        let memory = self.memory.clone();

        let stream = self.llm_chain.stream(input).await?;

        let output_stream = stream! {
            pin_mut!(stream);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(data) => {
                        let mut complete_ai_message_clone =
                            complete_ai_message_clone.lock().await;
                        complete_ai_message_clone.push_str(&data.content);

                        yield Ok(data);
                    },
                    Err(e) => {
                        yield Err(e);
                    }
                }
            }

            let mut memory = memory.write().await;
            memory.add_message(human_message);
            memory.add_ai_message(complete_ai_message.lock().await.to_string());
        };

        Ok(Box::pin(output_stream))
    }
}

impl<'a, I, O> GetPrompt<I::Target<'a>> for ConversationalChain<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'any> I::Target<'any>: Display,
    for<'any> O::Target<'any>: ChainOutput<I::Target<'any>>,
{
    fn get_prompt(&self, input: I::Target<'a>) -> Result<Prompt, TemplateError> {
        self.llm_chain
            .get_prompt(&ConversationalChainInput::new(input))
    }
}

#[cfg(test)]
mod tests {
    use async_openai::config::OpenAIConfig;

    use crate::{
        llm::openai::{OpenAI, OpenAIModel},
        schemas::DefaultChainInput,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_invoke_conversational() {
        let llm: OpenAI<OpenAIConfig> = OpenAI::builder()
            .with_model(OpenAIModel::Gpt35.to_string())
            .build();
        let chain: ConversationalChain = ConversationalChain::builder()
            .llm(llm)
            .build()
            .expect("Error building ConversationalChain");

        let input = DefaultChainInput::new("Soy de peru");
        // Execute the first `chain.invoke` and assert that it should succeed
        let result_first = chain.call(input).await;
        assert!(
            result_first.is_ok(),
            "Error invoking LLMChain: {:?}",
            result_first.err()
        );

        // Optionally, if you want to print the successful result, you can do so like this:
        if let Ok(result) = result_first {
            println!("Result: {:?}", result);
        }

        let input = DefaultChainInput::new("Cuales son platos tipicos de mi pais");
        // Execute the second `chain.invoke` and assert that it should succeed
        let result_second = chain.call(input).await;
        assert!(
            result_second.is_ok(),
            "Error invoking LLMChain: {:?}",
            result_second.err()
        );

        // Optionally, if you want to print the successful result, you can do so like this:
        if let Ok(result) = result_second {
            println!("Result: {:?}", result);
        }
    }
}
