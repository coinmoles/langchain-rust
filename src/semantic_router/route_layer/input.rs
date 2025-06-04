use std::{borrow::Cow, collections::HashMap};

use crate::schemas::{ChainInput, ChainInputCtor, TextReplacements};

pub struct RouteLayerInputCtor;
impl ChainInputCtor for RouteLayerInputCtor {
    type Target<'a> = RouteLayerInput<'a>;
}

#[derive(Clone)]
pub struct RouteLayerInput<'a> {
    pub description: Cow<'a, str>,
    pub query: Cow<'a, str>,
}

impl ChainInput for RouteLayerInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("description", self.description.as_ref().into()),
            ("query", self.query.as_ref().into()),
        ])
    }
}
