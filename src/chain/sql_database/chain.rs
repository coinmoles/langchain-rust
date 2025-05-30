use std::{error::Error, pin::Pin};

use async_trait::async_trait;
use futures::Stream;

use crate::{
    chain::{Chain, ChainError, LLMChain},
    schemas::{IntoWithUsage, Prompt, StreamData, TokenUsage, WithUsage},
    tools::SQLDatabase,
};

use super::{
    SQLDatabaseChainBuilder, SqlChainDefaultInput, SqlChainDefaultInputCtor, SqlChainLLMChainInput,
    SqlChainLLMChainInputCtor, QUERY_PREFIX_WITH, STOP_WORD,
};

pub struct SqlChainPromptBuilder {
    query: String,
}
impl SqlChainPromptBuilder {
    pub fn new() -> Self {
        Self {
            query: "".to_string(),
        }
    }

    pub fn query<S: Into<String>>(mut self, input: S) -> Self {
        self.query = input.into();
        self
    }
}

impl Default for SqlChainPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SQLDatabaseChain {
    pub(crate) llm_chain: LLMChain<SqlChainLLMChainInputCtor>,
    pub(crate) top_k: usize,
    pub(crate) database: SQLDatabase,
}

/// SQLChain let you interact with a db in human lenguage
///
/// The input variable name is `query`.
/// Example
/// ```rust,ignore
/// # async {
/// let options = ChainCallOptions::default();
/// let llm = OpenAI::default();
///
/// let db = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
/// let engine = PostgreSQLEngine::new(&db).await.unwrap();
/// let db = SQLDatabaseBuilder::new(engine).build().await.unwrap();
/// let chain = SQLDatabaseChainBuilder::new()
///     .llm(llm)
///     .top_k(4)
///     .database(db)
///     .options(options)
///     .build()
///     .expect("Failed to build LLMChain");
///
/// let input_variables = prompt_args! {
///     "query" => "Whats the phone number of luis"
///   };
///   //OR
/// let input_variables = chain.prompt_builder()
///     .query("Whats the phone number of luis")
///     .build();
/// match chain.invoke(input_variables).await {
///    Ok(result) => {
///     println!("Result: {:?}", result);
/// }
/// Err(e) => panic!("Error invoking LLMChain: {:?}", e),
/// }
///
/// }
/// ```
impl SQLDatabaseChain {
    pub fn builder<'b>() -> SQLDatabaseChainBuilder<'b> {
        SQLDatabaseChainBuilder::new()
    }

    pub fn prompt_builder(&self) -> SqlChainPromptBuilder {
        SqlChainPromptBuilder::new()
    }

    async fn build_input(
        &self,
        input: &SqlChainDefaultInput<'_>,
    ) -> Result<SqlChainLLMChainInput, ChainError> {
        let llm_input = format!("{}{}", input.query, QUERY_PREFIX_WITH);
        let tables_info = self
            .database
            .table_info(input.tables)
            .await
            .map_err(|e| ChainError::DatabaseError(e.to_string()))?;

        let llm_inputs = SqlChainLLMChainInput {
            input: llm_input.clone().into(),
            top_k: self.top_k,
            dialect: self.database.dialect().to_string().into(),
            tables_info: tables_info.into(),
        };

        Ok(llm_inputs)
    }

    async fn call_builder_chains(
        &self,
        input: &SqlChainDefaultInput<'_>,
    ) -> Result<(SqlChainLLMChainInput, Option<TokenUsage>), ChainError> {
        let mut token_usage: Option<TokenUsage> = None;

        let mut llm_inputs = self.build_input(input).await?;

        let output = self.llm_chain.call(&llm_inputs).await?;

        if let Some(tokens) = output.usage {
            token_usage = Some(tokens);
        }

        let query_result = self
            .database
            .query(&output.content)
            .await
            .map_err(|e| ChainError::DatabaseError(e.to_string()))?;

        llm_inputs.input = format!(
            "{}{}{}{}",
            llm_inputs.input, output.content, STOP_WORD, query_result,
        )
        .into();

        Ok((llm_inputs, token_usage))
    }
}

#[async_trait]
impl Chain for SQLDatabaseChain {
    type InputCtor = SqlChainDefaultInputCtor;
    type Output = String;

    async fn call<'i, 'i2>(
        &self,
        input: &'i SqlChainDefaultInput<'i2>,
    ) -> Result<WithUsage<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
        let (llm_inputs, token_usage) = self.call_builder_chains(input).await?;
        let output = self.llm_chain.call(&llm_inputs).await?;

        let total_usage = TokenUsage::merge_options([&output.usage, &token_usage]);

        let strs: Vec<&str> = output
            .content
            .split("\n\n")
            .next()
            .unwrap_or("")
            .split("Answer:")
            .collect();
        let output = if strs.len() > 1 { strs[1] } else { strs[0] };
        let output = output.trim().to_string();
        Ok(output.with_usage(total_usage))
    }

    async fn stream<'i, 'i2>(
        &self,
        input: &'i SqlChainDefaultInput<'i2>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    where
        'i: 'i2,
    {
        let (mut llm_inputs, _) = self.call_builder_chains(input).await?;

        self.llm_chain.stream(&mut llm_inputs).await
    }

    fn get_prompt<'i, 'i2>(
        &self,
        _input: &'i SqlChainDefaultInput<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2,
    {
        log::warn!("SQLDatabaseChain does not support get_prompt, returning an empty prompt.");
        Ok(Prompt::new(vec![]))
    }
}
