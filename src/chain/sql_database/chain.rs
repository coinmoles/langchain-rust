use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::{
    chain::{Chain, ChainError, LLMChain},
    schemas::{IntoWithUsage, StreamData, StringCtor, TokenUsage, WithUsage},
    tools::SQLDatabase,
};

use super::{
    SQLDatabaseChainBuilder, SqlChainInput, SqlChainInputCtor, SqlChainLLMChainInput,
    SqlChainLLMChainInputCtor, QUERY_PREFIX_WITH, STOP_WORD,
};

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

    pub fn prompt_builder(&self) -> SqlChainInput {
        SqlChainInput::default()
    }

    async fn build_input(
        &self,
        input: &SqlChainInput<'_>,
    ) -> Result<SqlChainLLMChainInput, ChainError> {
        let llm_input = format!("{}{}", input.query, QUERY_PREFIX_WITH);
        let tables_info = self
            .database
            .table_info(input.tables)
            .await
            .map_err(|e| ChainError::OtherError(format!("Database error: {e}")))?;

        let llm_inputs = SqlChainLLMChainInput {
            input: llm_input.clone().into(),
            top_k: self.top_k,
            dialect: self.database.dialect().to_string().into(),
            tables_info: tables_info.into(),
        };

        Ok(llm_inputs)
    }

    // TODO: Remove clone
    async fn call_builder_chains(
        &self,
        input: &SqlChainInput<'_>,
    ) -> Result<(SqlChainLLMChainInput, Option<TokenUsage>), ChainError> {
        let mut token_usage: Option<TokenUsage> = None;

        let builder_input = self.build_input(input).await?;
        let mut llm_input = builder_input.clone();
        let output = self.llm_chain.call(builder_input).await?;

        if let Some(tokens) = output.usage {
            token_usage = Some(tokens);
        }

        let query_result = self
            .database
            .query(&output.content)
            .await
            .map_err(|e| ChainError::OtherError(format!("Database error: {e}")))?;

        llm_input.input = format!(
            "{}{}{}{}",
            llm_input.input, output.content, STOP_WORD, query_result,
        )
        .into();

        Ok((llm_input, token_usage))
    }
}

#[async_trait]
impl Chain<SqlChainInputCtor, StringCtor> for SQLDatabaseChain {
    async fn call<'a>(&self, input: SqlChainInput<'a>) -> Result<WithUsage<String>, ChainError> {
        let (llm_inputs, token_usage) = self.call_builder_chains(&input).await?;
        let output = self.llm_chain.call(llm_inputs).await?;

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

    async fn stream(
        &self,
        input: SqlChainInput<'_>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamData, ChainError>> + Send>>, ChainError>
    {
        let (llm_inputs, _) = self.call_builder_chains(&input).await?;

        self.llm_chain.stream(llm_inputs).await
    }
}
