use std::borrow::Cow;

use crate::schemas::{ChainInput, Ctor};

#[derive(Clone, ChainInput, Ctor)]
pub struct RouteLayerInput<'a> {
    #[chain_input(text)]
    pub description: Cow<'a, str>,
    #[chain_input(text)]
    pub query: Cow<'a, str>,
}
