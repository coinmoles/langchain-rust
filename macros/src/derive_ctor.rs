use proc_macro_error::Diagnostic;
use quote::{format_ident, quote};

use crate::{
    attr::{LangchainStructAttrs, extract_attr, get_chain_struct_attrs},
    crate_path::default_crate_path,
};

pub fn derive_ctor(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let LangchainStructAttrs { crate_path, .. } =
        extract_attr(&input.attrs, get_chain_struct_attrs)?;
    let crate_path = crate_path.unwrap_or_else(default_crate_path);
    let target_lifetime = input.generics.lifetimes().next().map(|_| quote! { <'a> });

    let expanded = quote! {
        pub struct #ctor_struct_name;
        #[automatically_derived]
        impl #crate_path::schemas::Ctor for #ctor_struct_name
        {
            type Target<'a> = #struct_name #target_lifetime;
        }
    };

    Ok(expanded)
}
