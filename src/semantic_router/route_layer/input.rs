use std::{borrow::Cow, collections::HashMap};

use crate::schemas::{InputVariable, InputVariableCtor, TextReplacements};

pub struct RouteLayerInputCtor;
impl InputVariableCtor for RouteLayerInputCtor {
    type InputVariable<'a> = RouteLayerInput<'a>;
}

pub struct RouteLayerInput<'a> {
    pub description: Cow<'a, str>,
    pub query: Cow<'a, str>,
}

impl InputVariable for RouteLayerInput<'_> {
    fn text_replacements(&self) -> TextReplacements {
        HashMap::from([
            ("description", self.description.as_ref().into()),
            ("query", self.query.as_ref().into()),
        ])
    }
}
