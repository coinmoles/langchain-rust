use std::borrow::Cow;

use async_trait::async_trait;

use crate::{
    chain::{Chain, ChainError, ChainImpl},
    schemas::{
        ChainInputCtor, ChainOutput, IntoWithUsage, OutputTrace, Prompt, TokenUsage, WithUsage,
    },
};

pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: ChainInputCtor,
    M1: ChainOutput,
    for<'b> M2: ChainInputCtor<Target<'b> = M1>,
    O: ChainOutput,
{
    pub first: Box<dyn Chain<InputCtor = I, Output = M1> + 'a>,
    pub second: Box<dyn Chain<InputCtor = M2, Output = O> + 'a>,
}

#[async_trait]
impl<I, M1, M2, O> ChainImpl for SequentialChain<'_, I, M1, M2, O>
where
    I: ChainInputCtor,
    M1: ChainOutput,
    for<'a> M2: ChainInputCtor<Target<'a> = M1>,
    O: ChainOutput,
{
    type InputCtor = I;
    type Output = O;

    async fn call_impl<'i>(
        &self,
        input: Cow<'i, I::Target<'i>>,
    ) -> Result<WithUsage<Self::Output>, ChainError> {
        let result1 = self.first.call_impl(input).await?;
        let result2 = self.second.call(&result1.content).await?;

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
            .call_with_trace(&result1.final_step.content)
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
    // use crate::{
    //     chain::{LLMChain, LLMChainBuilder},
    //     llm::openai::OpenAI,
    //     schemas::{ChainInput, MessageType, TextReplacements},
    //     sequential_chain,
    //     template::MessageTemplate,
    // };

    // #[tokio::test]
    // #[ignore]
    // async fn test_sequential() {
    //     struct FirstInput<'a> {
    //         input: &'a str,
    //     }
    //     impl InputVariable for FirstInput<'_> {
    //         fn text_replacements(&self) -> TextReplacements {
    //             std::collections::HashMap::from([("input", self.input.into())])
    //         }
    //     }

    //     let llm = OpenAI::default();
    //     let chain1 = LLMChain::builder()
    //         .prompt(MessageTemplate::from_fstring(
    //             MessageType::HumanMessage,
    //             "dame un nombre para una tienda de {input}",
    //         ))
    //         .llm(llm.clone())
    //         .build()
    //         .expect("Failed to build LLMChain");

    //     let chain2 = LLMChain::builder()
    //         .prompt(MessageTemplate::from_fstring(
    //             MessageType::HumanMessage,
    //             "dame un slogan para una tienda llamada {nombre},tiene que incluir la palabra {palabra}",
    //         ))
    //         .llm(llm.clone())
    //         .build()
    //         .expect("Failed to build LLMChain");

    //     let chain = sequential_chain!(chain1, chain2);
    //     let result = chain
    //         .execute(
    //             &mut text_replacements! {
    //                 "input" => "medias",
    //                 "palabra" => "arroz"
    //             }
    //             .into(),
    //         )
    //         .await;
    //     assert!(
    //         result.is_ok(),
    //         "Expected `chain.call` to succeed, but it failed with error: {:?}",
    //         result.err()
    //     );

    //     if let Ok(output) = result {
    //         println!("{:?}", output);
    //     }
    // }
}
