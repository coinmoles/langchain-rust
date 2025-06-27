use async_trait::async_trait;

use crate::{
    chain::{InputCtor, OutputCtor},
    output_parser::OutputParseError,
};

#[async_trait]
pub trait OutputParser<I: InputCtor, O: OutputCtor>: Send + Sync {
    fn parse_from_text<'a>(&self, output: String) -> Result<O::Target<'a>, OutputParseError>;

    fn parse_from_text_and_input<'a>(
        &self,
        input: I::Target<'a>,
        text: String,
    ) -> Result<O::Target<'a>, (I::Target<'a>, OutputParseError)> {
        self.parse_from_text(text).map_err(|e| (input, e))
    }
}

pub trait ParseResultExt<T, E> {
    fn with_input<I>(self, input: I) -> Result<(I, T), (I, E)>;
}

impl<T, E> ParseResultExt<T, E> for Result<T, E> {
    fn with_input<I>(self, input: I) -> Result<(I, T), (I, E)> {
        match self {
            Ok(value) => Ok((input, value)),
            Err(err) => Err((input, err)),
        }
    }
}
