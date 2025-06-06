use std::{collections::HashSet, error::Error, pin::Pin, sync::Arc};

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
        messages::Message, GenerateResult, GenerateResultContent, InputVariables, Prompt,
        StreamData,
    },
};

use super::{ConversationalChainBuilder, ConversationalChainPromptBuilder};

pub struct ConversationalChain {
    pub(super) llm: LLMChain,
    pub(super) input_key: String,
    pub memory: Arc<RwLock<dyn Memory>>,
}

//Conversational Chain is a simple chain to interact with ai as a string of messages
impl ConversationalChain {
    pub fn builder<'a, 'b>() -> ConversationalChainBuilder<'a, 'b> {
        ConversationalChainBuilder::new()
    }

    pub fn prompt_builder(&self) -> ConversationalChainPromptBuilder {
        ConversationalChainPromptBuilder::new()
    }
}

#[async_trait]
impl Chain for ConversationalChain {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let input_variable = &input_variables
            .get_text_replacement(&self.input_key)
            .ok_or(ChainError::MissingInputVariable(self.input_key.clone()))?;
        let human_message = Message::new_human_message(input_variable);

        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };
        input_variables.insert_text_replacement("history", history);
        let result = self.llm.call(input_variables).await?;

        let mut memory = self.memory.write().await;
        memory.add_message(human_message);

        match &result.content {
            GenerateResultContent::Text(text) => memory.add_ai_message(text.clone()),
            GenerateResultContent::ToolCall(tool_calls) => {
                memory.add_tool_call_message(tool_calls.clone())
            }
            GenerateResultContent::Refusal(refusal) => {
                return Err(LLMError::OtherError(refusal.into()).into())
            }
        }
        Ok(result)
    }

    async fn stream(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let input_variable = input_variables
            .get_text_replacement(&self.input_key)
            .ok_or(ChainError::MissingInputVariable(self.input_key.clone()))?;
        let human_message = Message::new_human_message(input_variable);

        let history = {
            let memory = self.memory.read().await;
            memory.to_string()
        };

        input_variables.insert_text_replacement("history", history);

        let complete_ai_message = Arc::new(Mutex::new(String::new()));
        let complete_ai_message_clone = complete_ai_message.clone();

        let memory = self.memory.clone();

        let stream = self.llm.stream(input_variables).await?;
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

    fn get_input_keys(&self) -> HashSet<String> {
        [self.input_key.clone()].into_iter().collect()
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>> {
        self.llm.get_prompt(inputs)
    }
}

#[cfg(test)]
mod tests {
    use async_openai::config::OpenAIConfig;

    use crate::{
        chain::conversational::builder::ConversationalChainBuilder,
        llm::openai::{OpenAI, OpenAIModel},
        text_replacements,
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

        let mut input_variables_first: InputVariables = text_replacements! {
            "input" => "Soy de peru",
        }
        .into();
        // Execute the first `chain.invoke` and assert that it should succeed
        let result_first = chain.invoke(&mut input_variables_first).await;
        assert!(
            result_first.is_ok(),
            "Error invoking LLMChain: {:?}",
            result_first.err()
        );

        // Optionally, if you want to print the successful result, you can do so like this:
        if let Ok(result) = result_first {
            println!("Result: {:?}", result);
        }

        let mut input_variables_second = text_replacements! {
            "input" => "Cuales son platos tipicos de mi pais",
        }
        .into();
        // Execute the second `chain.invoke` and assert that it should succeed
        let result_second = chain.invoke(&mut input_variables_second).await;
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
