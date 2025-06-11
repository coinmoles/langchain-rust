use proc_macro_error::Diagnostic;
use quote::{format_ident, quote};

pub fn derive_ctor(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let target_lifetime = input.generics.lifetimes().next().map(|_| quote! { <'a> });

    let expanded = quote! {
        pub struct #ctor_struct_name;
        #[automatically_derived]
        impl Ctor for #ctor_struct_name
        {
            type Target<'a> = #struct_name #target_lifetime;
        }
    };

    Ok(expanded)
}
