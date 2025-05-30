use std::{error::Error, fmt::Display, pin::Pin, sync::Arc};

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
        messages::Message, DefaultChainInputCtor, InputVariableCtor, IntoWithUsage, LLMOutput,
        OutputVariable, Prompt, StreamData, WithUsage,
    },
};

use super::{ConversationalChainBuilder, ConversationalChainInput, ConversationalChainInputCtor};

pub struct ConversationalChain<I = DefaultChainInputCtor, O = String>
where
    I: InputVariableCtor,
    O: OutputVariable,
    for<'a> I::InputVariable<'a>: Display,
{
    pub(super) llm_chain: LLMChain<ConversationalChainInputCtor<I>, O>,
    pub(super) input_key: String,
    pub memory: Arc<RwLock<dyn Memory>>,
}

//Conversational Chain is a simple chain to interact with ai as a string of messages
impl<I, O> ConversationalChain<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
    for<'a> I::InputVariable<'a>: Display,
{
    pub fn builder<'a, 'b>() -> ConversationalChainBuilder<'a, 'b> {
        ConversationalChainBuilder::new()
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };
        let new_inptu = ConversationalChainInput::new(input).with_history(history);

        let stream = self.llm_chain.stream(&new_inptu).await?;

        todo!()
    }
}

#[async_trait]
impl<I, O> Chain for ConversationalChain<I, O>
where
    I: InputVariableCtor,
    O: OutputVariable,
    for<'a> I::InputVariable<'a>: Display,
{
    type InputCtor = I;
    type Output = O;

    async fn call<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<WithUsage<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
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

        Ok(O::try_from_string(result.content.into_text()?)?.with_usage(result.usage))
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        // let human_message = Message::new_human_message(input.to_string());

        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };
        let input = ConversationalChainInput::new(input).with_history(history);

        // let complete_ai_message = Arc::new(Mutex::new(String::new()));
        // let complete_ai_message_clone = complete_ai_message.clone();

        // let memory = self.memory.clone();

        let stream = self.llm_chain.stream(&input).await?;

        todo!()
        // let output_stream = stream! {
        //     pin_mut!(stream);
        //     while let Some(result) = stream.next().await {
        //         match result {
        //             Ok(data) => {
        //                 let mut complete_ai_message_clone =
        //                     complete_ai_message_clone.lock().await;
        //                 complete_ai_message_clone.push_str(&data.content);

        //                 yield Ok(data);
        //             },
        //             Err(e) => {
        //                 yield Err(e);
        //             }
        //         }
        //     }

        //     let mut memory = memory.write().await;
        //     memory.add_message(human_message);
        //     memory.add_ai_message(complete_ai_message.lock().await.to_string());
        // };

        // Ok(Box::pin(output_stream))
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2,
    {
        let input = ConversationalChainInput::new(input);
        let prompt = self.llm_chain.get_prompt(&input);

        prompt
    }
}

#[cfg(test)]
mod tests {
    use async_openai::config::OpenAIConfig;

    use crate::{
        chain::conversational::builder::ConversationalChainBuilder,
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
        let chain = ConversationalChainBuilder::new()
            .llm(llm)
            .build()
            .expect("Error building ConversationalChain");

        let input = DefaultChainInput::new("Soy de peru");
        // Execute the first `chain.invoke` and assert that it should succeed
        let result_first = chain.call(&input).await;
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
        let result_second = chain.call(&input).await;
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
