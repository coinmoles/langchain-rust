use async_trait::async_trait;
use serde::Serialize;

use crate::{
    chain::{Chain, ChainError},
    schemas::{InputCtor, IntoWithUsage, OutputCtor, OutputTrace, Prompt, TokenUsage, WithUsage},
};

pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    for<'b> M1::Target<'b>: Serialize + Clone + Into<M2::Target<'b>>,
    for<'b> O::Target<'b>: Serialize,
{
    pub first: Box<dyn Chain<InputCtor = I, OutputCtor = M1> + 'a>,
    pub second: Box<dyn Chain<InputCtor = M2, OutputCtor = O> + 'a>,
}

#[async_trait]
impl<I, M1, M2, O> Chain for SequentialChain<'_, I, M1, M2, O>
where
    I: InputCtor,
    M1: OutputCtor,
    M2: InputCtor,
    O: OutputCtor,
    for<'b> M1::Target<'b>: Serialize + Clone + Into<M2::Target<'b>>,
    for<'b> O::Target<'b>: Serialize,
{
    type InputCtor = I;
    type OutputCtor = O;

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

        let result = result1.extend(result2)?;

        Ok(result)
    }

    fn get_prompt(&self, input: I::Target<'_>) -> Result<Prompt, ChainError> {
        self.first.get_prompt(input)
    }
}

#[macro_export]
macro_rules! sequential_chain {
    ( $( $chain:expr ),* $(,)? ) => {
        {
            let chain = $crate::chain::SequentialChainBuilder::new();
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
        chain::LLMChain,
        llm::openai::OpenAI,
        schemas::{ChainInput, ChainOutput, Ctor, MessageType, TryFromStringError},
        sequential_chain,
        template::MessageTemplate,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_sequential() {
        #[derive(Debug, Clone, ChainInput, Ctor)]
        pub struct FirstInput<'a> {
            #[chain_input(text)]
            input: Cow<'a, str>,
            #[chain_input(text)]
            other: Cow<'a, str>,
        }
        #[derive(Debug, Clone, Serialize, ChainInput, Ctor)]
        pub struct SecondInput<'a> {
            #[chain_input(text)]
            nombre: Cow<'a, str>,
            #[chain_input(text)]
            other: Cow<'a, str>,
        }
        impl<'a> ChainOutput<FirstInput<'a>> for SecondInput<'a> {
            fn try_from_string(
                input: FirstInput<'a>,
                s: impl Into<String>,
            ) -> Result<Self, TryFromStringError> {
                let original: String = s.into();
                Ok(Self {
                    nombre: original.into(),
                    other: input.other,
                })
            }
        }

        let llm = OpenAI::default();
        let chain1: LLMChain<FirstInputCtor, SecondInputCtor> = LLMChain::builder()
            .prompt(MessageTemplate::from_fstring(
                MessageType::HumanMessage,
                "dame un nombre para una tienda de {input}",
            ))
            .llm(llm.clone())
            .build()
            .expect("Failed to build LLMChain");

        let chain2: LLMChain<SecondInputCtor> = LLMChain::builder()
            .prompt(MessageTemplate::from_fstring(
                MessageType::HumanMessage,
                "dame un slogan para una tienda llamada {nombre}",
            ))
            .llm(llm.clone())
            .build()
            .expect("Failed to build LLMChain");

        let chain = sequential_chain!(chain1, chain2);
        let result = chain
            .call(FirstInput {
                input: "zapatos".into(),
                other: "algo".into(),
            })
            .await;
        assert!(
            result.is_ok(),
            "Expected `chain.call` to succeed, but it failed with error: {:?}",
            result.err()
        );

        if let Ok(output) = result {
            println!("{:?}", output);
        }
    }
}
