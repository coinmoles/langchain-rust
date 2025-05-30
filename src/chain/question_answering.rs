use std::{borrow::Cow, collections::HashMap, error::Error, pin::Pin};

use crate::{
    language_models::llm::LLM,
    schemas::{messages::Message, Document, MessageType, Prompt, StreamData, WithUsage},
    schemas::{InputVariable, InputVariableCtor, TextReplacements},
    template::MessageTemplate,
};
use async_trait::async_trait;
use futures::Stream;
use indoc::indoc;

use super::{Chain, ChainError, LLMChain};

pub struct CondenseQuestionPromptConstructor;
impl InputVariableCtor for CondenseQuestionPromptConstructor {
    type InputVariable<'a> = CondenseQuestionPrompt<'a>;
}

pub struct CondenseQuestionPrompt<'a> {
    chat_history: Cow<'a, str>,
    question: Cow<'a, str>,
}

impl<'a> CondenseQuestionPrompt<'a> {
    pub fn new() -> Self {
        Self {
            chat_history: "".into(),
            question: "".into(),
        }
    }

    pub fn question(mut self, question: impl Into<Cow<'a, str>>) -> Self {
        self.question = question.into();
        self
    }

    pub fn chat_history(mut self, chat_history: &[Message]) -> Self {
        self.chat_history = Message::messages_to_string(chat_history).into();
        self
    }
}

impl InputVariable for CondenseQuestionPrompt<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("chat_history", self.chat_history.as_ref().into()),
            ("question", self.question.as_ref().into()),
        ])
    }
}

impl<'a> Default for CondenseQuestionPrompt<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CondenseQuestionGeneratorChain<I = CondenseQuestionPromptConstructor>
where
    I: InputVariableCtor,
{
    chain: LLMChain<I>,
}

impl CondenseQuestionGeneratorChain<CondenseQuestionPromptConstructor> {
    pub fn new<L: Into<Box<dyn LLM>>>(llm: L) -> Self {
        let condense_question_prompt_template = MessageTemplate::from_jinja2(
            MessageType::SystemMessage,
            indoc! {"
            Given the following conversation and a follow up question, rephrase the follow up question to be a standalone question, in its original language.

            Chat History:
            {{chat_history}}
            Follow Up Input: {{question}}
            Standalone question:"},
        );

        let chain = LLMChain::builder()
            .llm(llm)
            .prompt(condense_question_prompt_template)
            .build()
            .unwrap(); //Its safe to unwrap here because we are sure that the prompt and the LLM are
                       //set.
        Self { chain }
    }

    pub fn prompt_builder(&self) -> CondenseQuestionPrompt {
        CondenseQuestionPrompt::new()
    }
}

#[async_trait]
impl<I> Chain for CondenseQuestionGeneratorChain<I>
where
    I: InputVariableCtor,
{
    type InputCtor = I;
    type Output = String;

    async fn call<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<WithUsage<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
        self.chain.call(input).await
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        self.chain.stream(input).await
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2,
    {
        self.chain.get_prompt(input)
    }
}

pub struct StuffQACtor;
impl InputVariableCtor for StuffQACtor {
    type InputVariable<'a> = StuffQA<'a>;
}

pub struct StuffQA<'a> {
    input_documents: Vec<Document>,
    question: Cow<'a, str>,
}

impl<'a> StuffQA<'a> {
    pub fn new() -> Self {
        Self {
            input_documents: vec![],
            question: "".into(),
        }
    }

    pub fn documents(mut self, documents: &[Document]) -> Self {
        self.input_documents = documents.to_vec();
        self
    }

    pub fn question(mut self, question: impl Into<Cow<'a, str>>) -> Self {
        self.question = question.into();
        self
    }
}

impl InputVariable for StuffQA<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("question", self.question.as_ref().into()),
            (
                "context",
                self.input_documents
                    .iter()
                    .map(|doc| doc.page_content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
                    .into(),
            ),
        ])
    }
}

impl<'a> Default for StuffQA<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{
        chain::{Chain, StuffDocument, StuffQA},
        llm::openai::OpenAI,
        schemas::Document,
    };

    #[tokio::test]
    #[ignore]
    async fn test_qa() {
        let llm = OpenAI::default();
        let chain = StuffDocument::load_stuff_qa(llm);
        let input = StuffQA::new()
            .documents(&[
                Document::new(indoc! {"
                    Question: Which is the favorite text editor of luis
                    Answer: Nvim"
                }),
                Document::new(indoc! {"
                    Question: How old is Luis
                    Answer: 24"
                }),
            ])
            .question("How old is luis and whats his favorite text editor")
            .into();

        let ouput = chain.call(&input).await.unwrap().content;

        println!("{}", ouput);
    }
}
