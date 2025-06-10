use proc_macro::TokenStream;
use proc_macro_error::{ResultExt, proc_macro_error};
use syn::{DeriveInput, parse_macro_input};

mod attr;
mod check_type;
mod crate_path;
mod derive_chain_input;
mod derive_chain_output;
mod derive_ctor;
mod helpers;
mod rename;

#[proc_macro_error]
#[proc_macro_derive(ChainInput, attributes(langchain, serde))]
pub fn derive_chain_input(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_chain_input::derive_chain_input(input)
        .unwrap_or_abort()
        .into()
}

#[proc_macro_error]
#[proc_macro_derive(ChainOutput, attributes(langchain, serde))]
pub fn derive_chain_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_chain_output::derive_chain_output(input)
        .unwrap_or_abort()
        .into()
}

#[proc_macro_derive(Ctor, attributes(langchain))]
pub fn derive_ctor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_ctor::derive_ctor(input).unwrap_or_abort().into()
}
