use async_trait::async_trait;

use crate::chain::{InputCtor, OutputCtor};
use crate::utils::parse::ParseError;

#[async_trait]
pub trait OutputParser<I: InputCtor, O: OutputCtor>: Send + Sync {
    fn parse_from_text<'a>(&self, output: String) -> Result<O::Target<'a>, ParseError>;

    fn parse_from_text_and_input<'a>(
        &self,
        input: I::Target<'a>,
        text: String,
    ) -> Result<O::Target<'a>, (I::Target<'a>, ParseError)> {
        self.parse_from_text(text).map_err(|e| (input, e))
    }
}
