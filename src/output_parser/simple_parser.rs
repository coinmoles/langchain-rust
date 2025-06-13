use async_trait::async_trait;

use crate::{
    output_parser::OutputParseError,
    schemas::{ChainOutput, InputCtor, OutputCtor},
};

use super::OutputParser;

pub struct SimpleParser<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    trim: bool,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O> SimpleParser<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }
}

impl<I, O> Default for SimpleParser<I, O>
where
    I: InputCtor,
    O: OutputCtor,
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    fn default() -> Self {
        Self {
            trim: true,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<I: InputCtor, O: OutputCtor> OutputParser<I, O> for SimpleParser<I, O>
where
    for<'a> O::Target<'a>: ChainOutput<I::Target<'a>>,
{
    fn parse_from_text_and_input<'a>(
        &self,
        input: I::Target<'a>,
        output: String,
    ) -> Result<O::Target<'a>, OutputParseError> {
        if self.trim {
            O::Target::construct_from_text_and_input(input, output.trim())
        } else {
            O::Target::construct_from_text_and_input(input, output)
        }
    }

    fn parse_from_text<'a>(&self, output: String) -> Result<O::Target<'a>, OutputParseError> {
        if self.trim {
            O::Target::construct_from_text(output.trim())
        } else {
            O::Target::construct_from_text(output)
        }
    }
}
