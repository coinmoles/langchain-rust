use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::{
    attr::is_text_input_attr,
    field::{generate_text_replacement, get_fields},
};

mod attr;
mod check_type;
mod field;

#[proc_macro_derive(ChainInput, attributes(input))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).named;

    let text_fields = fields
        .iter()
        .filter(|f| f.attrs.iter().any(is_text_input_attr));
    let text_replacements = text_fields.map(generate_text_replacement);

    let expanded = quote! {
        impl ChainInput for #struct_name<'_> {
            fn text_replacements<'a>(&'a self) -> std::collections::HashMap<&'a str, Cow<'a, str>> {
                std::collections::HashMap::from([
                    #(#text_replacements),*
                ])
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(ChainInputCtor, attributes(input))]
pub fn derive_chain_input_ctor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let expanded = quote! {
        pub struct #ctor_struct_name;
        #[automatically_derived]
        impl ChainInputCtor for #ctor_struct_name {
            type Target<'a> = #struct_name<'a>;
        }
    };

    expanded.into()
}
