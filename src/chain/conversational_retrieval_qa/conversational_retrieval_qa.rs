use futures::Stream;
use futures_util::{pin_mut, StreamExt};
use std::{collections::HashSet, error::Error, pin::Pin, sync::Arc};

use async_stream::stream;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::{
    chain::{Chain, ChainError, CondenseQuestionPromptBuilder, StuffQABuilder, DEFAULT_RESULT_KEY},
    schemas::{
        BaseMemory, InputVariables, Message, MessageType, Retriever, StreamData,
        {GenerateResult, TokenUsage},
    },
};
// _conversationalRetrievalQADefaultInputKey             = "question"
// _conversationalRetrievalQADefaultSourceDocumentKey    = "source_documents"
// 	_conversationalRetrievalQADefaultGeneratedQuestionKey = "generated_question"
// )

const CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_SOURCE_DOCUMENT_KEY: &str = "source_documents";
const CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_GENERATED_QUESTION_KEY: &str = "generated_question";

pub struct ConversationalRetrieverChain {
    pub(crate) retriever: Box<dyn Retriever>,
    pub memory: Arc<RwLock<dyn BaseMemory>>,
    pub(crate) combine_documents_chain: Box<dyn Chain>,
    pub(crate) condense_question_chain: Box<dyn Chain>,
    pub(crate) rephrase_question: bool,
    pub(crate) return_source_documents: bool,
    pub(crate) input_key: String,  //Default is `question`
    pub(crate) output_key: String, //default is output
}

impl ConversationalRetrieverChain {
    async fn get_question(
        &self,
        history: &[Message],
        input: &str,
    ) -> Result<(String, Option<TokenUsage>), ChainError> {
        if history.is_empty() {
            return Ok((input.to_string(), None));
        }
        let mut token_usage: Option<TokenUsage> = None;
        let question = match self.rephrase_question {
            true => {
                let result = self
                    .condense_question_chain
                    .call(
                        &mut CondenseQuestionPromptBuilder::new()
                            .question(input)
                            .chat_history(history)
                            .build()
                            .into(),
                    )
                    .await?;

                if let Some(usage) = result.usage {
                    token_usage = Some(usage);
                };
            }
            false => input,
        };

        Ok((question.into(), token_usage))
    }
}

#[async_trait]
impl Chain for ConversationalRetrieverChain {
    async fn call(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<GenerateResult, ChainError> {
        let output = self.execute(input_variables).await?;
        let result: GenerateResult = serde_json::from_value(output[DEFAULT_RESULT_KEY].clone())?;
        Ok(result)
    }

    // async fn execute(
    //     &self,
    //     input_variables: &mut InputVariables,
    // ) -> Result<HashMap<String, Value>, ChainError> {
    //     let mut token_usage: Option<TokenUsage> = None;
    //     let input_variable = &input_variables
    //         .get_text_replacement(&self.input_key)
    //         .ok_or(ChainError::MissingInputVariable(self.input_key.clone()))?;

    //     let human_message = Message::new(MessageType::HumanMessage, input_variable);
    //     let history = {
    //         let memory = self.memory.lock().await;
    //         memory.messages()
    //     };

    //     let (question, token) = self.get_question(&history, &human_message.content).await?;
    //     if let Some(token) = token {
    //         token_usage = Some(token);
    //     }

    //     let documents = self
    //         .retriever
    //         .get_relevant_documents(&question)
    //         .await
    //         .map_err(|e| ChainError::RetrieverError(e.to_string()))?;

    //     let mut output = self
    //         .combine_documents_chain
    //         .call(
    //             &mut StuffQABuilder::new()
    //                 .documents(&documents)
    //                 .question(question.clone())
    //                 .build()
    //                 .into(),
    //         )
    //         .await?;

    //     if let Some(tokens) = &output.usage {
    //         if let Some(mut token_usage) = token_usage {
    //             token_usage.add(tokens);
    //             output.usage = Some(token_usage)
    //         }
    //     }

    //     {
    //         let mut memory = self.memory.lock().await;
    //         memory.add_message(human_message);
    //         memory.add_message(Message::new(MessageType::AIMessage, &output.generation));
    //     }

    //     let mut result = HashMap::new();
    //     result.insert(self.output_key.clone(), json!(output.generation));

    //     result.insert(DEFAULT_RESULT_KEY.to_string(), json!(output));

    //     if self.return_source_documents {
    //         result.insert(
    //             CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_SOURCE_DOCUMENT_KEY.to_string(),
    //             json!(documents),
    //         );
    //     }

    //     if self.rephrase_question {
    //         result.insert(
    //             CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_GENERATED_QUESTION_KEY.to_string(),
    //             json!(question),
    //         );
    //     }

    //     Ok(result)
    // }

    async fn stream(
        &self,
        input_variables: &mut InputVariables,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let input_variable = &input_variables
            .get_text_replacement(&self.input_key)
            .ok_or(ChainError::MissingInputVariable(self.input_key.clone()))?;

        let human_message = Message::new_human_message(input_variable);
        let history = {
            let memory = self.memory.lock().await;
            memory.messages()
        };

        let (question, _) = self.get_question(&history, &human_message.content).await?;

        let documents = self
            .retriever
            .get_relevant_documents(&question)
            .await
            .map_err(|e| ChainError::RetrieverError(e.to_string()))?;

        let stream = self
            .combine_documents_chain
            .stream(
                &mut StuffQABuilder::new()
                    .documents(&documents)
                    .question(question.clone())
                    .build()
                    .into(),
            )
            .await?;

        let memory = self.memory.clone();
        let complete_ai_message = Arc::new(Mutex::new(String::new()));
        let complete_ai_message_clone = complete_ai_message.clone();
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

            let mut memory = memory.lock().await;
            memory.add_message(human_message);
            memory.add_ai_message(complete_ai_message.lock().await.to_string());
        };

        Ok(Box::pin(output_stream))
    }

    fn get_input_keys(&self) -> HashSet<String> {
        [self.input_key.clone()].into_iter().collect()
    }

    fn get_output_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        if self.return_source_documents {
            keys.push(CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_SOURCE_DOCUMENT_KEY.to_string());
        }

        if self.rephrase_question {
            keys.push(CONVERSATIONAL_RETRIEVAL_QA_DEFAULT_GENERATED_QUESTION_KEY.to_string());
        }

        keys.push(self.output_key.clone());
        keys.push(DEFAULT_RESULT_KEY.to_string());

        keys
    }

    fn get_prompt(&self, inputs: &InputVariables) -> Result<Prompt, Box<dyn Error + Send + Sync>> {
        self.condense_question_chain.get_prompt(inputs)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use indoc::indoc;

    use crate::{
        chain::ConversationalRetrieverChainBuilder,
        llm::openai::{OpenAI, OpenAIModel},
        memory::SimpleMemory,
        schemas::Document,
    };

    use super::*;

    struct RetrieverTest {}
    #[async_trait]
    impl Retriever for RetrieverTest {
        async fn get_relevant_documents(
            &self,
            _question: &str,
        ) -> Result<Vec<Document>, Box<dyn Error>> {
            Ok(vec![
                Document::new(indoc! {"
                    Question: Which is the favorite text editor of luis
                    Answer: Nvim"
                }),
                Document::new(indoc! {"
                    Question: How old is luis
                    Answer: 24"
                }),
                Document::new(indoc! {"
                    Question: Where do luis live
                    Answer: Peru"
                }),
                Document::new(indoc! {"
                    Question: What's his favorite food
                    Answer: Pan con chicharron"
                }),
            ])
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_invoke_retriever_conversational() {
        let llm = OpenAI::default().with_model(OpenAIModel::Gpt35.to_string());
        let chain = ConversationalRetrieverChainBuilder::new()
            .llm(llm)
            .retriever(RetrieverTest {})
            .memory(SimpleMemory::new().into())
            .build()
            .expect("Error building ConversationalChain");

        let mut input_variables_first: InputVariables =
            StuffQABuilder::new().question("Hola").build().into();
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

        let mut input_variables_second: InputVariables = StuffQABuilder::new()
            .question("Cual es la comida favorita de luis")
            .build()
            .into();
        // Execute the second `chain.invoke` and assert that it should succeed
        let result_second = chain.invoke(&mut input_variables_second).await;
        assert!(
            result_second.is_ok(),
            "Error invoking LLMChain: {:?}",
            result_second.err()
        );

        if let Ok(result) = result_second {
            println!("Result: {:?}", result);
        }
    }
}
