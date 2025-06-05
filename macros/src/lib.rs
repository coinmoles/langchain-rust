use proc_macro::TokenStream;
use proc_macro_error::{ResultExt, proc_macro_error};
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::{
    attr::{ChainInputFieldAttr, extract_field_attrs, extract_struct_attrs},
    field::{generate_text_replacement, get_fields},
};

mod attr;
mod check_type;
mod field;
mod rename;

#[proc_macro_error]
#[proc_macro_derive(ChainInput, attributes(input, serde))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).unwrap_or_abort().named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let struct_attrs = extract_struct_attrs(&input.attrs).unwrap_or_abort();
    let fields_with_attrs = fields
        .into_iter()
        .filter_map(|f| {
            extract_field_attrs(&f.attrs)
                .unwrap_or_abort()
                .map(|attrs| (f, attrs))
        })
        .collect::<Vec<_>>();

    let inner_fields = fields_with_attrs
        .iter()
        .filter(|(_, attrs)| attrs.input == ChainInputFieldAttr::Inner);
    let inner_extensions = inner_fields.map(|(f, _)| {
        let ident = f.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.text_replacements()); }
    });

    let text_fields = fields_with_attrs
        .iter()
        .filter(|(_, attrs)| attrs.input == ChainInputFieldAttr::Text);
    let text_replacements =
        text_fields.map(|(field, attrs)| generate_text_replacement(field, attrs, &struct_attrs));

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ChainInput for #struct_name #ty_generics
        #where_clause
        {
            fn text_replacements<'b>(&'b self) -> std::collections::HashMap<&'b str, Cow<'b, str>> {
                let mut map = std::collections::HashMap::from([
                    #(#text_replacements),*
                ]);
                #(#inner_extensions)*;
                map
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(ChainInputCtor, attributes(input, serde))]
pub fn derive_chain_input_ctor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let target_lifetime = input.generics.lifetimes().next().map(|_| quote! { <'b> });

    let expanded = quote! {
        pub struct #ctor_struct_name;
        #[automatically_derived]
        impl ChainInputCtor for #ctor_struct_name
        {
            type Target<'b> = #struct_name #target_lifetime;
        }
    };

    expanded.into()
}
