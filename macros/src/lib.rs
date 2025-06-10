use proc_macro::TokenStream;
use proc_macro_error::{ResultExt, abort, abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input, spanned::Spanned};

use crate::{
    attr::{
        ChainInputFieldAttr, ChainOutputFieldAttr, extract_attr, extract_attr_default,
        get_chain_input_field_attr, get_chain_output_field_attr, get_chain_output_struct_attrs,
        get_chain_struct_attrs, get_serde_field_attrs, get_serde_struct_attrs,
    },
    chain_input::{generate_placeholder_replacement, generate_text_replacement, get_fields},
    chain_output::{deser_struct, field_initializers},
};

mod attr;
mod chain_input;
mod chain_output;
mod check_type;
mod crate_path;
mod rename;

#[proc_macro_error]
#[proc_macro_derive(ChainInput, attributes(chain, chain_input, serde))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).unwrap_or_abort().named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let serde_struct_attrs =
        extract_attr_default(&input.attrs, get_serde_struct_attrs).unwrap_or_abort();
    let chain_struct_attrs =
        extract_attr_default(&input.attrs, get_chain_struct_attrs).unwrap_or_abort();
    let fields_with_attrs = fields
        .into_iter()
        .filter_map(|f| {
            let chain_input_attr =
                extract_attr(&f.attrs, get_chain_input_field_attr).unwrap_or_abort();
            let serde_attrs =
                extract_attr_default(&f.attrs, get_serde_field_attrs).unwrap_or_abort();

            chain_input_attr.map(|c| (f, c, serde_attrs))
        })
        .collect::<Vec<_>>();

    let rename_all = serde_struct_attrs.rename_all;
    let crate_path = chain_struct_attrs.crate_path;

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

#[proc_macro_error]
#[proc_macro_derive(ChainOutput, attributes(chain, chain_output, serde))]
pub fn derive_chain_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).unwrap_or_abort().named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let serde_struct_attrs =
        extract_attr_default(&input.attrs, get_serde_struct_attrs).unwrap_or_abort();
    let chain_struct_attrs =
        extract_attr_default(&input.attrs, get_chain_struct_attrs).unwrap_or_abort();
    let chain_output_struct_attrs =
        extract_attr_default(&input.attrs, get_chain_output_struct_attrs).unwrap_or_abort();
    let fields_with_attrs = fields
        .into_iter()
        .map(|f| {
            let chain_output_attr =
                extract_attr_default(&f.attrs, get_chain_output_field_attr).unwrap_or_abort();
            let serde_attrs =
                extract_attr_default(&f.attrs, get_serde_field_attrs).unwrap_or_abort();
            (f, chain_output_attr, serde_attrs)
        })
        .collect::<Vec<_>>();

    if let Some((f, _, _)) = {
        fields_with_attrs
            .iter()
            .filter(|(_, chain_output_attr, _)| {
                *chain_output_attr == ChainOutputFieldAttr::Response
            })
            .nth(1)
    } {
        abort!(
            f.span(),
            "ChainOutput struct cannot have more than one field with `#[chain_output(from_response)]` attribute"
        );
    }

    let rename_all = serde_struct_attrs.rename_all;
    let crate_path = chain_struct_attrs.crate_path;
    let serde_path = chain_struct_attrs.serde_path;
    let serde_json_path = chain_struct_attrs.serde_json_path;
    let input_struct = chain_output_struct_attrs.input.unwrap_or_else(|| {
        abort_call_site!("ChainOutput struct must have a `chain_output(input = \"...\")` attribute")
    });

    let deser_struct = deser_struct(&fields_with_attrs, &serde_path, &rename_all);
    let field_initializers = field_initializers(&fields_with_attrs, &rename_all);

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ChainOutput<#input_struct> for #struct_name #ty_generics
        #where_clause
        {
            fn parse_output(input: #input_struct, output: impl Into<String>) -> Result<Self, #crate_path::schemas::OutputParseError> {
                #deser_struct

                let original: String = output.into();
                let deserialized = match #serde_json_path::from_str::<InputDeserialize>(&original) {
                    Ok(deserialized) => deserialized,
                    Err(e) => {
                        return Err(#crate_path::schemas::OutputParseError {
                            original,
                            error: Some(Box::new(e)),
                        });
                    }
                };
                Ok(Self {
                    #(#field_initializers),*
                })
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Ctor, attributes(chain, chain_input))]
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
