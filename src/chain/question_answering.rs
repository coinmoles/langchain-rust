use std::{borrow::Cow, collections::HashMap, pin::Pin};

use crate::{
    chain::{Chain, ChainInput, Ctor, InputCtor, StringCtor, TextReplacements},
    llm::LLM,
    schemas::{messages::Message, Document, MessageType, StreamData, WithUsage},
    template::MessageTemplate,
};
use async_trait::async_trait;
use futures::Stream;
use indoc::indoc;

use super::{ChainError, LLMChain};

#[derive(Clone, ChainInput, Ctor)]
pub struct CondenseQuestionPrompt<'a> {
    #[langchain(into = "text")]
    chat_history: Cow<'a, str>,
    #[langchain(into = "text")]
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

impl<'a> Default for CondenseQuestionPrompt<'a> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CondenseQuestionGeneratorChain<I: InputCtor = CondenseQuestionPromptCtor> {
    chain: LLMChain<I>,
}

impl CondenseQuestionGeneratorChain<CondenseQuestionPromptCtor> {
    pub fn new<L: Into<Box<dyn LLM>>>(llm: L) -> Self {
        let condense_question_prompt_template = MessageTemplate::from_jinja2(
            MessageType::System,
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
impl<I: InputCtor> Chain<I, StringCtor> for CondenseQuestionGeneratorChain<I> {
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<String>, ChainError> {
        self.chain.call(input).await
    }

    async fn stream(
        &self,
        input: I::Target<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        self.chain.stream(input).await
    }
}

#[derive(Clone, Ctor)]
pub struct StuffQA<'a> {
    input_documents: Cow<'a, [Document]>,
    question: Cow<'a, str>,
}

impl<'a> StuffQA<'a> {
    pub fn new() -> Self {
        Self {
            input_documents: Cow::Borrowed(&[]),
            question: "".into(),
        }
    }

    pub fn documents(mut self, documents: impl Into<Cow<'a, [Document]>>) -> Self {
        self.input_documents = documents.into();
        self
    }

    pub fn question(mut self, question: impl Into<Cow<'a, str>>) -> Self {
        self.question = question.into();
        self
    }
}

impl ChainInput for StuffQA<'_> {
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
        let documents = [
            Document::new(indoc! {"
                    Question: Which is the favorite text editor of luis
                    Answer: Nvim"
            }),
            Document::new(indoc! {"
                    Question: How old is Luis
                    Answer: 24"
            }),
        ];
        let input = StuffQA::new()
            .documents(&documents)
            .question("How old is luis and whats his favorite text editor");

        let output = chain.call(input).await.unwrap().content;

        println!("{}", output);
    }
}
