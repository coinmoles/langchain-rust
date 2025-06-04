use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed};

use crate::{
    attr::extract_serde_rename,
    check_type::{extract_option_inner_type, is_cow_str_type, is_str_type, is_string_type},
    rename::RenameAll,
};

pub fn get_fields(input: &DeriveInput) -> &FieldsNamed {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named,
            _ => panic!("ChainInput can only be derived for structs with named fields"),
        },
        _ => panic!("ChainInput can only be derived for structs"),
    }
}

fn generate_text_replacement_conversion(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();

    if let Some(ty) = extract_option_inner_type(&field.ty) {
        if is_str_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.as_deref().unwrap_or("")) };
        } else if is_string_type(ty) || is_cow_str_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.as_ref().map_or("", |s| s.as_ref())) };
        } else {
            return quote! { std::borrow::Cow::Owned(self.#ident.map_or_else(|| String::new(), |s| s.to_string())) };
        }
    }

    if is_str_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(self.#ident) }
    } else if is_string_type(&field.ty) || is_cow_str_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(self.#ident.as_ref()) }
    } else {
        quote! { std::borrow::Cow::Owned(self.#ident.to_string()) }
    }
}

pub fn generate_text_replacement(
    field: &Field,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let key = match extract_serde_rename(&field.attrs) {
        Some(rename) => rename,
        None => {
            let ident = field.ident.as_ref().unwrap();
            match rename_all {
                Some(rename_all) => rename_all.apply(ident.to_string()),
                None => ident.to_string(),
            }
        }
    };
    let value = generate_text_replacement_conversion(field);

    quote! {
        (#key, #value)
    }
}
