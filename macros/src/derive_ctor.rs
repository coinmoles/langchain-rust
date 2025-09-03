use proc_macro_error2::Diagnostic;
use quote::{format_ident, quote};
use syn::{WherePredicate, punctuated::Punctuated};

use crate::{
    attr::{LangchainStructAttrs, extract_attr, get_chain_struct_attrs},
    crate_path::default_crate_path,
};

fn is_chain_input_bound(tb: &syn::TraitBound) -> bool {
    tb.path
        .segments
        .last()
        .is_some_and(|seg| seg.ident == "ChainInput")
}

fn has_chain_input_bound<'a>(mut bounds: impl Iterator<Item = &'a syn::TypeParamBound>) -> bool {
    bounds.any(|bound| match bound {
        syn::TypeParamBound::Trait(tb) => is_chain_input_bound(tb),
        _ => false,
    })
}

fn type_params_with_chain_input_bound(generics: &syn::Generics) -> Vec<&syn::Ident> {
    let result = generics
        .type_params()
        .filter(|param| has_chain_input_bound(param.bounds.iter()))
        .map(|param| &param.ident)
        .collect::<Vec<_>>();

    let Some(where_clause) = &generics.where_clause else {
        return result;
    };

    where_clause
        .predicates
        .iter()
        .fold(result, |mut acc, clause| {
            let WherePredicate::Type(syn::PredicateType {
                bounded_ty, bounds, ..
            }) = clause
            else {
                return acc;
            };
            let syn::Type::Path(type_path) = &bounded_ty else {
                return acc;
            };
            let Some(ident) = type_path.path.get_ident() else {
                return acc;
            };
            if has_chain_input_bound(bounds.iter()) {
                acc.push(ident);
            }
            acc
        })
}

fn into_ctor_generics(
    crate_path: &syn::Path,
    generics: syn::Generics,
) -> (syn::Generics, Option<proc_macro2::TokenStream>) {
    let filtered_params = generics
        .params
        .into_iter()
        .filter(|param| !matches!(param, syn::GenericParam::Lifetime(_)))
        .map(|param| match param {
            syn::GenericParam::Type(mut type_param) => {
                type_param.bounds.iter_mut().for_each(|bound| {
                    let syn::TypeParamBound::Trait(tb) = bound else {
                        return;
                    };
                    if is_chain_input_bound(tb) {
                        tb.path = syn::parse2(quote! { #crate_path::chain::InputCtor }).unwrap();
                        type_param.default = None;
                    }
                });
                syn::GenericParam::Type(type_param)
            }
            other => other,
        })
        .collect::<Punctuated<_, syn::token::Comma>>();
    let has_generics = !filtered_params.is_empty();

    let ctor_generics = syn::Generics {
        lt_token: has_generics.then(Default::default),
        gt_token: has_generics.then(Default::default),
        params: filtered_params,
        where_clause: generics.where_clause,
    };

    let phantom_data = has_generics.then(|| {
        let phantom_data_generics = ctor_generics
            .type_params()
            .map(|param| &param.ident)
            .collect::<Vec<_>>();

        quote! {
            (std::marker::PhantomData<(#(#phantom_data_generics),*)>)
        }
    });

    (ctor_generics, phantom_data)
}

fn construct_target_generics(generics: &syn::Generics) -> proc_macro2::TokenStream {
    let type_params_with_chain_input_bound = type_params_with_chain_input_bound(generics);
    let generic_params = generics.params.iter().map(|param| match param {
        syn::GenericParam::Type(type_param) => {
            let ident = &type_param.ident;
            if type_params_with_chain_input_bound.contains(&&type_param.ident) {
                quote! { #ident::Target<'a> }
            } else {
                quote! { #ident }
            }
        }
        syn::GenericParam::Const(const_param) => {
            let ident = &const_param.ident;
            quote! { #ident }
        }
        syn::GenericParam::Lifetime(_) => {
            quote! { 'a }
        }
    });

    quote! { <#(#generic_params),*> }
}

pub fn derive_ctor(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let struct_name = &input.ident;
    let ctor_struct_name = format_ident!("{struct_name}Ctor");

    let LangchainStructAttrs { crate_path, .. } =
        extract_attr(&input.attrs, get_chain_struct_attrs)?;
    let crate_path = crate_path.unwrap_or_else(default_crate_path);

    let target_generics = construct_target_generics(&input.generics);
    let (ctor_generics, phantom_data) = into_ctor_generics(&crate_path, input.generics);
    let (impl_generics, ty_generics, where_clause) = ctor_generics.split_for_impl();

    let expanded = quote! {
        pub struct #ctor_struct_name #ctor_generics #phantom_data;
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::Ctor for #ctor_struct_name #ty_generics #where_clause
        {
            type Target<'a> = #struct_name #target_generics;
        }
    };

    Ok(expanded)
}
