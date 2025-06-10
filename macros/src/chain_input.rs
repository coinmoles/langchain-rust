use proc_macro_error::abort;
use quote::{ToTokens, quote};
use syn::{Field, LitStr, spanned::Spanned};

use crate::{
    check_type::{
        extract_option_inner_type, is_cow_str_type, is_message_slice_type, is_str_type,
        is_string_type, is_vec_message_type,
    },
    helpers::get_renamed_key,
    rename::RenameAll,
};

fn generate_text_replacement_conversion(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();

    if let Some(ty) = extract_option_inner_type(&field.ty) {
        if is_str_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.unwrap_or("")) };
        } else if is_string_type(ty) || is_cow_str_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.as_deref().unwrap_or("")) };
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
    rename: &Option<LitStr>,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let key = get_renamed_key(field, rename, rename_all);
    let value = generate_text_replacement_conversion(field);

    quote! {
        (#key, #value)
    }
}

fn generate_placeholder_replacement_conversion(field: &Field) -> proc_macro2::TokenStream {
    let ident = field.ident.as_ref().unwrap();

    if let Some(ty) = extract_option_inner_type(&field.ty) {
        if is_message_slice_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.unwrap_or(&[])) };
        } else if is_vec_message_type(ty) || is_cow_str_type(ty) {
            return quote! { std::borrow::Cow::Borrowed(self.#ident.as_deref().unwrap_or(&[])) };
        } else {
            abort!(
                ty.span(),
                "Unsupported type for placeholder replacement: {}",
                ty.to_token_stream()
            );
        }
    }

    if is_message_slice_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(self.#ident) }
    } else if is_vec_message_type(&field.ty) || is_cow_str_type(&field.ty) {
        quote! { std::borrow::Cow::Borrowed(&self.#ident) }
    } else {
        abort!(
            field.ty.span(),
            "Unsupported type for placeholder replacement: {}",
            field.ty.to_token_stream()
        );
    }
}

pub fn generate_placeholder_replacement(
    field: &Field,
    rename: &Option<LitStr>,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let key = get_renamed_key(field, rename, rename_all);
    let value = generate_placeholder_replacement_conversion(field);

    quote! {
        (#key, #value)
    }
}
