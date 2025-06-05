use std::borrow::Cow;

use async_trait::async_trait;

use crate::{
    chain::{Chain, ChainError, ChainImpl},
    schemas::{
        AsInput, ChainInputCtor, ChainOutput, IntoWithUsage, OutputTrace, Prompt, TokenUsage,
        WithUsage,
    },
};

pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: ChainInputCtor,
    M1: ChainOutput + AsInput,
    for<'b> M2: ChainInputCtor<Target<'b> = M1::AsInput<'b>>,
    O: ChainOutput,
{
    pub first: Box<dyn Chain<InputCtor = I, Output = M1> + 'a>,
    pub second: Box<dyn Chain<InputCtor = M2, Output = O> + 'a>,
}

#[async_trait]
impl<I, M1, M2, O> ChainImpl for SequentialChain<'_, I, M1, M2, O>
where
    I: ChainInputCtor,
    M1: ChainOutput + AsInput,
    for<'a> M2: ChainInputCtor<Target<'a> = M1::AsInput<'a>>,
    O: ChainOutput,
{
    type InputCtor = I;
    type Output = O;

    async fn call_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<WithUsage<Self::Output>, ChainError> {
        let result1 = self.first.call_impl(input).await?;
        let result2 = self.second.call_owned(result1.content.as_input()).await?;

        let usage = TokenUsage::merge_options([&result1.usage, &result2.usage]);
        Ok(result2.content.with_usage(usage))
    }

    async fn call_with_trace_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<OutputTrace<Self::Output>, ChainError> {
        let result1 = self.first.call_with_trace_impl(input).await?;
        let result2 = self
            .second
            .call_with_trace_owned(result1.final_step.content.as_input())
            .await?;

        let result = result1.extend(result2)?;

        Ok(result)
    }

    fn get_prompt_impl<'i>(&self, input: Cow<'i, I::Target<'i>>) -> Result<Prompt, ChainError> {
        self.first.get_prompt_impl(input)
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
        chain::{Chain, LLMChain},
        llm::openai::OpenAI,
        schemas::{
            AsInput, ChainInput, ChainInputCtor, ChainOutput, MessageType, TryFromStringError,
        },
        sequential_chain,
        template::MessageTemplate,
    };

    #[tokio::test]
    #[ignore]
    async fn test_sequential() {
        #[derive(Debug, Clone, ChainInput, ChainInputCtor)]
        pub struct FirstInput<'a> {
            #[chain_input(text)]
            input: Cow<'a, str>,
        }
        #[derive(Debug, Clone, Serialize, ChainInput, ChainInputCtor)]
        pub struct SecondInput<'a> {
            #[chain_input(text)]
            nombre: Cow<'a, str>,
        }
        impl ChainOutput for SecondInput<'_> {
            fn try_from_string(s: impl Into<String>) -> Result<Self, TryFromStringError> {
                let original: String = s.into();
                Ok(Self {
                    nombre: original.into(),
                })
            }
        }
        impl AsInput for SecondInput<'_> {
            type AsInput<'a>
                = SecondInput<'a>
            where
                Self: 'a;

            fn as_input(&self) -> Self::AsInput<'_> {
                SecondInput {
                    nombre: self.nombre.as_ref().into(),
                }
            }
        }

        let llm = OpenAI::default();
        let chain1: LLMChain<FirstInputCtor, SecondInput> = LLMChain::builder()
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
            .call(&FirstInput {
                input: "zapatos".into(),
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
