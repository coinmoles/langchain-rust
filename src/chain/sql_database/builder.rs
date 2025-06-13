use crate::{
    chain::{LLMChain, SqlChainLLMChainInputCtor},
    language_models::{llm::LLM, options::CallOptions},
    output_parser::OutputParser,
    prompt_template,
    schemas::{BuilderError, MessageType, StringCtor},
    template::{MessageTemplate, PromptTemplate},
    tools::SQLDatabase,
};

use super::{
    chain::SQLDatabaseChain,
    prompt::{DEFAULT_SQLSUFFIX, DEFAULT_SQLTEMPLATE},
    STOP_WORD,
};

pub struct SQLDatabaseChainBuilder<'b> {
    llm: Option<Box<dyn LLM>>,
    top_k: Option<usize>,
    database: Option<SQLDatabase>,
    output_key: Option<&'b str>,
    output_parser: Option<Box<dyn OutputParser<SqlChainLLMChainInputCtor, StringCtor>>>,
}

impl<'b> SQLDatabaseChainBuilder<'b> {
    pub(super) fn new() -> Self {
        Self {
            llm: None,
            top_k: None,
            database: None,
            output_key: None,
            output_parser: None,
        }
    }

    pub fn llm(mut self, llm: impl Into<Box<dyn LLM>>) -> Self {
        self.llm = Some(llm.into());
        self
    }

    pub fn output_key<S: Into<String>>(
        mut self,
        output_key: &'b (impl AsRef<str> + ?Sized),
    ) -> Self {
        self.output_key = Some(output_key.as_ref());
        self
    }

    pub fn output_parser(
        mut self,
        output_parser: impl Into<Box<dyn OutputParser<SqlChainLLMChainInputCtor, StringCtor>>>,
    ) -> Self {
        self.output_parser = Some(output_parser.into());
        self
    }

    pub fn top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn database(mut self, database: SQLDatabase) -> Self {
        self.database = Some(database);
        self
    }

    pub fn build(self) -> Result<SQLDatabaseChain, BuilderError> {
        let llm = self.llm.ok_or(BuilderError::MissingField("llm"))?;
        let top_k = self.top_k.ok_or(BuilderError::MissingField("top_k"))?;
        let database = self
            .database
            .ok_or(BuilderError::MissingField("database"))?;

        let prompt: PromptTemplate = prompt_template![MessageTemplate::from_jinja2(
            MessageType::Human,
            format!("{DEFAULT_SQLTEMPLATE}{DEFAULT_SQLSUFFIX}"),
        )];

        let llm_chain = {
            let mut llm = llm.clone_box();
            llm.add_call_options(CallOptions::new().with_stop_words(vec![STOP_WORD.to_string()]));

            let mut builder = LLMChain::builder().prompt(prompt).llm(llm);

            if let Some(output_parser) = self.output_parser {
                builder = builder.output_parser(output_parser);
            }

            builder
                .build()
                .map_err(|e| BuilderError::Inner("llm_chain", Box::new(e)))?
        };

        Ok(SQLDatabaseChain {
            llm_chain,
            top_k,
            database,
        })
    }
}
