use std::error::Error;

use async_trait::async_trait;

use crate::{
    chain::{Chain, ChainError},
    schemas::{
        InputVariableCtor, IntoWithUsage, OutputTrace, OutputVariable, Prompt, TokenUsage,
        WithUsage,
    },
};

pub struct SequentialChain<'a, I, M1, M2, O>
where
    I: InputVariableCtor,
    M1: OutputVariable,
    for<'b> M2: InputVariableCtor<InputVariable<'b> = M1>,
    O: OutputVariable,
{
    pub first: Box<dyn Chain<InputCtor = I, Output = M1> + 'a>,
    pub second: Box<dyn Chain<InputCtor = M2, Output = O> + 'a>,
}

#[async_trait]
impl<I, M1, M2, O> Chain for SequentialChain<'_, I, M1, M2, O>
where
    I: InputVariableCtor,
    M1: OutputVariable,
    for<'a> M2: InputVariableCtor<InputVariable<'a> = M1>,
    O: OutputVariable,
{
    type InputCtor = I;
    type Output = O;

    async fn call<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<WithUsage<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
        let result1 = self.first.call(input).await?;
        let result2 = self.second.call(&result1.content).await?;

        let usage = TokenUsage::merge_options([&result1.usage, &result2.usage]);
        Ok(result2.content.with_usage(usage))
    }

    async fn call_with_trace<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<OutputTrace<Self::Output>, ChainError>
    where
        'i: 'i2,
    {
        let result1 = self.first.call_with_trace(&input).await?;
        let result2 = self
            .second
            .call_with_trace(&result1.final_step.content)
            .await?;

        let result = result1.extend(result2)?;

        Ok(result)
    }

    fn get_prompt<'i, 'i2>(
        &self,
        input: &'i I::InputVariable<'i2>,
    ) -> Result<Prompt, Box<dyn Error + Send + Sync>>
    where
        'i: 'i2,
    {
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
    use crate::{
        chain::{LLMChain, LLMChainBuilder},
        llm::openai::OpenAI,
        schemas::{InputVariable, MessageType, TextReplacements},
        sequential_chain,
        template::MessageTemplate,
    };

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
