use std::borrow::Cow;

use crate::schemas::{ChainInput, ChainInputCtor};

#[derive(Clone, ChainInput, ChainInputCtor)]
pub struct RouteLayerInput<'a> {
    #[input(text)]
    pub description: Cow<'a, str>,
    #[input(text)]
    pub query: Cow<'a, str>,
}
