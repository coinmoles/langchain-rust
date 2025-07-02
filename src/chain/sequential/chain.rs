use async_trait::async_trait;
use serde::Serialize;

use crate::{
    chain::{Chain, ChainError, InputCtor, OutputCtor},
    schemas::{IntoWithUsage, OutputTrace, TokenUsage, WithUsage},
};

pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    for<'any> M1::Target<'any>: Serialize + Clone + Into<M2::Target<'any>>,
    for<'any> O::Target<'any>: Serialize,
{
    pub first: Box<dyn Chain<I, M1> + 'a>,
    pub second: Box<dyn Chain<M2, O> + 'a>,
}

#[async_trait]
impl<I, M1, M2, O> Chain<I, O> for SequentialChain<'_, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    for<'any> M1::Target<'any>: Serialize + Clone + Into<M2::Target<'any>>,
    for<'any> O::Target<'any>: Serialize,
{
    async fn call<'a>(&self, input: I::Target<'a>) -> Result<WithUsage<O::Target<'a>>, ChainError> {
        let result1 = self.first.call(input).await?;
        let result2 = self.second.call(result1.content.into()).await?;

        let usage = TokenUsage::merge_options([&result1.usage, &result2.usage]);
        Ok(result2.content.with_usage(usage))
    }

    async fn call_with_trace<'a>(
        &self,
        input: I::Target<'a>,
    ) -> Result<OutputTrace<O::Target<'a>>, ChainError> {
        let result1 = self.first.call_with_trace(input).await?;
        let result2 = self
            .second
            .call_with_trace(result1.final_step.content.clone().into())
            .await?;

        let result = result1.extend(result2);

        Ok(result)
    }
}

#[macro_export]
macro_rules! sequential_chain {
    () => {
        $crate::chain::EmptyChain
    };
    ( $first:expr $(, $chain:expr )* $(,)? ) => {
        {
            let chain = $first;
            $(
                let chain = $crate::chain::AddChain::add_chain(chain, $chain);
            )*
            chain
        }
    };
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use serde::Serialize;

    use crate::{
        chain::{ChainInput, ChainOutput, Ctor, LLMChain},
        llm::openai::OpenAI,
        schemas::MessageType,
        sequential_chain,
        template::MessageTemplate,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_sequential() {
        #[derive(Debug, Clone, ChainInput, Ctor)]
        pub struct Chain1Input<'a> {
            #[langchain(into = "text")]
            input: &'a str,
            palabra: &'a str,
        }
        #[derive(Debug, Clone, Serialize, ChainInput, ChainOutput, Ctor)]
        #[langchain(from_input = Chain1Input<'a>)]
        pub struct Chain2Input<'a> {
            #[langchain(from = "response", into = "text")]
            nombre: Cow<'a, str>,
            #[langchain(from = "input", into = "text")]
            palabra: Cow<'a, str>,
        }

        let llm = OpenAI::default();
        let chain1: LLMChain<Chain1InputCtor, Chain2InputCtor> = LLMChain::builder()
            .prompt(MessageTemplate::from_fstring(
                MessageType::Human,
                "dame un nombre para una tienda de {input}",
            ))
            .llm(llm.clone())
            .build()
            .expect("Failed to build LLMChain");

        let chain2: LLMChain<Chain2InputCtor> = LLMChain::builder()
            .prompt(MessageTemplate::from_fstring(
                MessageType::Human,
                "dame un slogan para una tienda llamada {nombre}, tiene que incluir la palabra {palabra}",
            ))
            .llm(llm.clone())
            .build()
            .expect("Failed to build LLMChain");

        let chain = sequential_chain!(chain1, chain2);
        let result = chain
            .call(Chain1Input {
                input: "medias",
                palabra: "arroz",
            })
            .await;
        assert!(
            result.is_ok(),
            "Expected `chain.call` to succeed, but it failed with error: {:?}",
            result.err()
        );

        if let Ok(output) = result {
            println!("{output:?}");
        }
    }
}
