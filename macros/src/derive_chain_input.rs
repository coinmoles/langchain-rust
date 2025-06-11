use proc_macro_error::{Diagnostic, abort};
use quote::{ToTokens, quote};
use syn::{Field, LitStr, spanned::Spanned};

use crate::{
    attr::{
        ChainInputKind, LangchainFieldAttrs, LangchainStructAttrs, SerdeFieldAttrs,
        SerdeStructAttrs, extract_attr, get_chain_struct_attrs, get_langchain_field_attrs,
        get_serde_field_attrs, get_serde_struct_attrs,
    },
    check_type::{
        extract_option_inner_type, is_cow_str_type, is_message_slice_type, is_str_type,
        is_string_type, is_vec_message_type,
    },
    crate_path::default_crate_path,
    helpers::{get_fields, get_renamed_key},
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

fn generate_text_replacement(
    field_spec: &ChainInputFieldSpec<'_>,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let key = get_renamed_key(field_spec.field, &field_spec.rename, rename_all);
    let value = generate_text_replacement_conversion(field_spec.field);

    quote! { (#key, #value) }
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

fn generate_placeholder_replacement(
    field_spec: &ChainInputFieldSpec<'_>,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let key = get_renamed_key(field_spec.field, &field_spec.rename, rename_all);
    let value = generate_placeholder_replacement_conversion(field_spec.field);

    quote! { (#key, #value) }
}

struct ChainInputFieldSpec<'a> {
    field: &'a syn::Field,
    input_kind: ChainInputKind,
    rename: Option<LitStr>,
}

impl<'a> ChainInputFieldSpec<'a> {
    fn try_new(
        field: &'a syn::Field,
        langchain_attrs: LangchainFieldAttrs,
        serde_attrs: SerdeFieldAttrs,
    ) -> Option<Self> {
        Some(Self {
            field,
            input_kind: langchain_attrs.input_kind?,
            rename: serde_attrs.rename,
        })
    }
}

struct ChainInputStructSpec {
    crate_path: syn::Path,
    rename_all: Option<RenameAll>,
}

impl ChainInputStructSpec {
    fn new(langchain_attrs: LangchainStructAttrs, serde_attrs: SerdeStructAttrs) -> Self {
        Self {
            crate_path: langchain_attrs
                .crate_path
                .unwrap_or_else(default_crate_path),
            rename_all: serde_attrs.rename_all,
        }
    }
}

pub fn derive_chain_input(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let struct_name = &input.ident;
    let fields = &get_fields(&input)?.named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let ChainInputStructSpec {
        crate_path,
        rename_all,
    } = ChainInputStructSpec::new(
        extract_attr(&input.attrs, get_chain_struct_attrs)?,
        extract_attr(&input.attrs, get_serde_struct_attrs)?,
    );
    let field_specs = fields
        .into_iter()
        .map(|field| -> Result<_, Diagnostic> {
            let langchain_attrs = extract_attr(&field.attrs, get_langchain_field_attrs)?;
            let serde_attrs = extract_attr(&field.attrs, get_serde_field_attrs)?;
            Ok(ChainInputFieldSpec::try_new(
                field,
                langchain_attrs,
                serde_attrs,
            ))
        })
        .collect::<Result<Vec<Option<_>>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let inner_fields = field_specs
        .iter()
        .filter(|f| f.input_kind == ChainInputKind::Inner)
        .collect::<Vec<_>>();

    let text_replacements = field_specs
        .iter()
        .filter(|f| f.input_kind == ChainInputKind::Text)
        .map(|f| generate_text_replacement(f, &rename_all));
    let inner_text_replacement_extensions = inner_fields.iter().map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.text_replacements()); }
    });

    let placeholder_replacements = field_specs
        .iter()
        .filter(|f| f.input_kind == ChainInputKind::Placeholder)
        .map(|f| generate_placeholder_replacement(f, &rename_all));
    let inner_placeholder_replacement_extensions = inner_fields.iter().map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.placeholder_replacements()); }
    });

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::schemas::ChainInput for #struct_name #ty_generics
        #where_clause
        {
            fn text_replacements(&self) -> #crate_path::schemas::TextReplacements {
                let mut map = std::collections::HashMap::from([
                    #(#text_replacements),*
                ]);
                #(#inner_text_replacement_extensions)*;
                map
            }

            fn placeholder_replacements(&self) -> #crate_path::schemas::PlaceholderReplacements {
                let mut map = std::collections::HashMap::from([
                    #(#placeholder_replacements),*
                ]);
                #(#inner_placeholder_replacement_extensions)*;
                map
            }
        }
    };

    Ok(expanded)
}
