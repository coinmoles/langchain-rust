use proc_macro::TokenStream;
use proc_macro_error::{ResultExt, proc_macro_error};
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::{
    attr::{
        ChainInputFieldAttr, extract_attr, get_chain_input_field_attr,
        get_chain_input_struct_attrs, get_serde_struct_attrs,
    },
    crate_path::default_crate_path,
    field::{generate_placeholder_replacement, generate_text_replacement, get_fields},
};

mod attr;
mod check_type;
mod crate_path;
mod field;
mod rename;

#[proc_macro_error]
#[proc_macro_derive(ChainInput, attributes(chain_input, serde))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).unwrap_or_abort().named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let serde_struct_attrs = extract_attr(&input.attrs, get_serde_struct_attrs)
        .unwrap_or_abort()
        .unwrap_or_default();
    let chain_input_struct_attrs = extract_attr(&input.attrs, get_chain_input_struct_attrs)
        .unwrap_or_abort()
        .unwrap_or_default();
    let fields_with_attrs = fields
        .into_iter()
        .filter_map(|f| {
            let chain_input_attr =
                extract_attr(&f.attrs, get_chain_input_field_attr).unwrap_or_abort();
            let serde_attrs = extract_attr(&f.attrs, crate::attr::get_serde_field_attrs)
                .unwrap_or_abort()
                .unwrap_or_default();

            chain_input_attr.map(|c| (f, c, serde_attrs))
        })
        .collect::<Vec<_>>();

    let rename_all = serde_struct_attrs.rename_all;
    let crate_path = chain_input_struct_attrs
        .crate_path
        .unwrap_or_else(default_crate_path);

    let inner_fields = fields_with_attrs
        .iter()
        .filter(|(_, chain_input_attr, _)| *chain_input_attr == ChainInputFieldAttr::Inner)
        .collect::<Vec<_>>();

    let text_fields = fields_with_attrs
        .iter()
        .filter(|(_, chain_input_attr, _)| *chain_input_attr == ChainInputFieldAttr::Text);
    let text_replacements = text_fields.map(|(field, _, serde_attrs)| {
        generate_text_replacement(field, &serde_attrs.rename, &rename_all)
    });
    let inner_text_replacement_extensions = inner_fields.iter().map(|(f, _, _)| {
        let ident = f.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.text_replacements()); }
    });

    let placeholder_fields = fields_with_attrs
        .iter()
        .filter(|(_, chain_input_attr, _)| *chain_input_attr == ChainInputFieldAttr::Placeholder);
    let placeholder_replacements = placeholder_fields.map(|(field, _, serde_attrs)| {
        generate_placeholder_replacement(field, &serde_attrs.rename, &rename_all)
    });
    let inner_placeholder_replacement_extensions = inner_fields.iter().map(|(f, _, _)| {
        let ident = f.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.placeholder_replacements()); }
    });

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ChainInput for #struct_name #ty_generics
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

    expanded.into()
}

#[proc_macro_derive(Ctor, attributes(chain_input, serde))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let target_lifetime = input.generics.lifetimes().next().map(|_| quote! { <'b> });

    let expanded = quote! {
        pub struct #ctor_struct_name;
        #[automatically_derived]
        impl Ctor for #ctor_struct_name
        {
            type Target #target_lifetime = #struct_name #target_lifetime;
        }
    };

    expanded.into()
}

// #[proc_macro_derive(ChainOutput, attributes(chain_output))]
// pub fn derive_chain_output(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);

//     let struct_name = &input.ident;
//     let fields = &get_fields(&input).unwrap_or_abort().named;
//     let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

//     let struct_attrs = extract_struct_attrs(&input.attrs).unwrap_or_abort();
//     let crate_path = &struct_attrs.crate_path;

//     let expanded = quote! {
//         #[automatically_derived]
//         impl #impl_generics ChainOutput for #struct_name #ty_generics
//         #where_clause
//         {
//             fn try_from_string(s: impl Into<String>) -> Result<Self, #crate_path::schemas::TryFromStringError> {
//                 let original: String = s.into();
//                 Ok(Self {

//                 })
//             }
//         }
//     };

//     expanded.into()
// }
