use std::borrow::Cow;

use crate::chain::{ChainInput, Ctor};

#[derive(Clone, ChainInput, Ctor)]
pub struct RouteLayerInput<'a> {
    #[langchain(into = "text")]
    pub description: Cow<'a, str>,
    #[langchain(into = "text")]
    pub query: Cow<'a, str>,
}
