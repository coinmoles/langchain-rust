use proc_macro_error::{Diagnostic, Level};
use syn::{Data, DeriveInput, Fields, FieldsNamed, LitStr};

use crate::rename::RenameAll;

pub fn get_fields(input: &DeriveInput) -> Result<&FieldsNamed, Diagnostic> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => Ok(fields_named),
            _ => Err(Diagnostic::new(
                Level::Error,
                "ChainInput can only be derived for structs with named fields".into(),
            )),
        },
        _ => Err(Diagnostic::new(
            Level::Error,
            "ChainInput can only be derived for structs".into(),
        )),
    }
}

pub fn get_renamed_key(
    field: &syn::Field,
    rename: &Option<LitStr>,
    rename_all: &Option<RenameAll>,
) -> String {
    match rename {
        Some(rename) => rename.value(),
        None => {
            let ident = field.ident.as_ref().unwrap();
            match rename_all {
                Some(rename_all) => rename_all.apply(ident.to_string()),
                None => ident.to_string(),
            }
        }
    }
}

pub(crate) trait BoolExt {
    // fn otherwise<T>(self, f: impl FnOnce() -> T) -> Option<T>;
    fn otherwise_some<T>(self, t: T) -> Option<T>;
}

impl BoolExt for bool {
    // fn otherwise<T>(self, f: impl FnOnce() -> T) -> Option<T> {
    //     if self { None } else { Some(f()) }
    // }

    fn otherwise_some<T>(self, t: T) -> Option<T> {
        if self { None } else { Some(t) }
    }
}
