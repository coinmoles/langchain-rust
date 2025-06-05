use proc_macro::TokenStream;
use proc_macro_error::{ResultExt, abort_call_site, proc_macro_error};
use quote::{format_ident, quote};
use syn::{DeriveInput, Lifetime, parse_macro_input};

use crate::{
    attr::{ChainInputFieldAttr, extract_field_attrs, extract_struct_attrs},
    check_type::{extract_option_inner_type, is_cow_type, is_str_type},
    field::{generate_placeholder_replacement, generate_text_replacement, get_fields},
};

mod attr;
mod check_type;
mod field;
mod rename;

#[proc_macro_error]
#[proc_macro_derive(ChainInput, attributes(chain_input, serde))]
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
    let crate_path = &struct_attrs.crate_path;

    let inner_fields = fields_with_attrs
        .iter()
        .filter(|(_, attrs)| attrs.input == ChainInputFieldAttr::Inner)
        .collect::<Vec<_>>();

    let text_fields = fields_with_attrs
        .iter()
        .filter(|(_, attrs)| attrs.input == ChainInputFieldAttr::Text);
    let text_replacements =
        text_fields.map(|(field, attrs)| generate_text_replacement(field, attrs, &struct_attrs));
    let inner_text_replacement_extensions = inner_fields.iter().map(|(f, _)| {
        let ident = f.ident.as_ref().unwrap();
        quote! { map.extend(self.#ident.text_replacements()); }
    });

    let placeholder_fields = fields_with_attrs
        .iter()
        .filter(|(_, attrs)| attrs.input == ChainInputFieldAttr::Placeholder);
    let placeholder_replacements = placeholder_fields
        .map(|(field, attrs)| generate_placeholder_replacement(field, attrs, &struct_attrs));
    let inner_placeholder_replacement_extensions = inner_fields.iter().map(|(f, _)| {
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

#[proc_macro_derive(ChainInputCtor, attributes(chain_input, serde))]
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

#[proc_macro_derive(AsInput, attributes(chain_input))]
pub fn derive_chain_output(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;
    let fields = &get_fields(&input).unwrap_or_abort().named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let input_lt: Lifetime = syn::parse2(quote! { 'input0 }).unwrap();
    let input_generics = {
        let mut cloned_generics = input.generics.clone();
        if let Some(lt) = cloned_generics.lifetimes_mut().next() {
            lt.lifetime = input_lt.clone();
        }
        cloned_generics
    };
    let input_ty_generics = input_generics.split_for_impl().1;

    let fields = fields
        .into_iter()
        .map(|f| {
            let ident = f.ident.as_ref().unwrap_or_else(|| {
                abort_call_site!("Tuple structs not supported");
            });
            let attrs = extract_field_attrs(&f.attrs).unwrap_or_abort();

            match attrs {
                Some(attrs) if attrs.input == ChainInputFieldAttr::Text => {
                    if let Some(ty) = extract_option_inner_type(&f.ty) {
                        if is_cow_type(ty) {
                            return quote! { #ident: self.#ident.map(|a| std::borrow::Cow::Borrowed(a.as_ref())) }
                        } else if is_str_type(ty) {
                            return quote! { #ident: self.#ident }
                        } else {
                            return quote! { #ident: self.#ident.clone() }
                        }
                    }

                    if is_cow_type(&f.ty) {
                        quote! { #ident: std::borrow::Cow::Borrowed(self.#ident.as_ref()) }
                    } else if is_str_type(&f.ty) {
                        quote! { #ident: self.#ident }
                    } else {
                        quote! { #ident: self.#ident.clone() }
                    }
                }
                _ => quote! { #ident: Default::default() },
            }
        });

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics AsInput for #struct_name #ty_generics
        #where_clause
        {
            type AsInput<#input_lt> = #struct_name #input_ty_generics
            where
                Self: #input_lt;

            fn as_input(&self) -> Self::AsInput<'_> {
                #struct_name {
                    #(#fields,)*
                }
            }
        }
    };

    expanded.into()
}
