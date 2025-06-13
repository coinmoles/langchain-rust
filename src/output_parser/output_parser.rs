use async_trait::async_trait;

use crate::{
    output_parser::OutputParseError,
    schemas::{InputCtor, OutputCtor},
};

#[async_trait]
pub trait OutputParser<I: InputCtor, O: OutputCtor>: Send + Sync {
    fn parse_from_text<'a>(&self, output: String) -> Result<O::Target<'a>, OutputParseError>;

    fn parse_from_text_and_input<'a>(
        &self,
        _input: I::Target<'a>,
        text: String,
    ) -> Result<O::Target<'a>, OutputParseError> {
        self.parse_from_text(text)
    }
}
