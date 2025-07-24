use async_trait::async_trait;
use regex::Regex;

use crate::{
    chain::{ChainOutput, InputCtor, OutputCtor},
    output_parser::ParseResultExt,
};

use super::{OutputParseError, OutputParser};

pub struct RegexParser<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    re: Regex,
    trim: bool,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> RegexParser<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    pub fn new(re: Regex) -> Self {
        Self {
            re,
            trim: true,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    pub fn code_block() -> Self {
        let re = Regex::new(r"```(?:\w+)?\s*([\s\S]+?)\s*```").expect("Static regex is valid");
        Self::new(re)
    }

    pub fn sanitize<'a>(&self, output: &'a str) -> Result<&'a str, OutputParseError> {
        let cap = self
            .re
            .captures(output)
            .ok_or_else(|| OutputParseError::Other("No match found".into()))?;

        let captured = cap
            .get(1)
            .ok_or_else(|| OutputParseError::Other("Failed to capture code block".into()))?
            .as_str();

        if self.trim {
            Ok(captured.trim())
        } else {
            Ok(captured)
        }
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> OutputParser<I, O> for RegexParser<I, O>
where
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    fn parse_from_text_and_input<'a>(
        &self,
        input: I::Target<'a>,
        output: String,
    ) -> Result<O::Target<'a>, (I::Target<'a>, OutputParseError)> {
        let (input, sanitized_output) = self.sanitize(&output).with_input(input)?;
        O::Target::from_text_and_input(input, sanitized_output)
    }

    fn parse_from_text<'a>(&self, output: String) -> Result<O::Target<'a>, OutputParseError> {
        O::Target::from_text(self.sanitize(&output)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::StringCtor;

    use super::*;

    #[tokio::test]
    async fn test_markdown_parser_finds_code_block() {
        let parser: RegexParser<(), StringCtor> = RegexParser::code_block();
        let markdown_content = r#"
```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
        let result = parser.parse_from_text_and_input((), markdown_content.into());
        println!("{result:?}");

        let correct = r#"fn main() {
    println!("Hello, world!");
}"#;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), correct);
    }
}
