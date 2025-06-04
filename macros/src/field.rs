use quote::quote;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed};

use crate::check_type::{is_cow_str_type, is_str_type, is_string_type};

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

    if is_str_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(self.#ident) }
    } else if is_string_type(&field.ty) || is_cow_str_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(self.#ident.as_ref()) }
    } else {
        quote! { std::borrow::Cow::Owned(self.#ident.to_string()) }
    }
}

pub fn generate_text_replacement(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let key = ident.to_string();
    let value = generate_text_replacement_conversion(field);

    quote! {
        (#key, #value)
    }
}
