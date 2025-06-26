use proc_macro_error::{Diagnostic, Level};
use quote::{ToTokens, format_ident, quote};
use syn::{LitStr, parse_quote_spanned, spanned::Spanned};

use crate::{
    attr::{
        ChainOutputSource, LangchainFieldAttrs, LangchainStructAttrs, SerdeFieldAttrs,
        SerdeStructAttrs, extract_attr, get_chain_struct_attrs, get_langchain_field_attrs,
        get_serde_field_attrs, get_serde_struct_attrs,
    },
    check_type::is_cow_str_type,
    crate_path::{default_crate_path, default_serde_json_path, default_serde_path},
    helpers::{get_fields, get_renamed_key},
    rename::RenameAll,
};

fn deser_struct(
    field_specs: &[ChainOutputFieldSpec<'_>],
    serde_path: &syn::Path,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let serde_path_str = serde_path.to_token_stream().to_string().replace(" ", "");

    let deser_fields = field_specs
        .iter()
        .filter(|f| f.output_source == ChainOutputSource::ResponseJson)
        .map(|f| {
            let ident = f.field.ident.as_ref().unwrap();
            let ty = if is_cow_str_type(&f.field.ty) {
                parse_quote_spanned! { f.field.ty.span() => String }
            } else {
                f.field.ty.clone()
            };

            let rename_attr = f.rename.as_ref().map(|r| {
                quote! {
                    #[serde(rename = #r)]
                }
            });

            quote! {
                #rename_attr
                pub #ident: #ty
            }
        });
    let rename_all_attr = rename_all.as_ref().map(|rename_all| {
        let rename_all = rename_all.to_string();
        quote! {
            #[serde(rename_all = #rename_all)]
        }
    });

    quote! {
        #[derive(#serde_path::Deserialize)]
        #[serde(crate = #serde_path_str)]
        #rename_all_attr
        struct InputDeserialize {
            #(#deser_fields),*
        }
    }
}

fn field_initializers(
    field_specs: &[ChainOutputFieldSpec<'_>],
    rename_all: &Option<RenameAll>,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    field_specs.iter().map(|f| {
        let ident = f.field.ident.as_ref().unwrap();
        let key = get_renamed_key(f.field, &f.rename, rename_all);

        match f.output_source {
            ChainOutputSource::Input => {
                let key_ident = format_ident!("{key}");
                quote! { #ident: input.#key_ident.into() }
            }
            ChainOutputSource::Response => quote! { #ident: original.into() },
            ChainOutputSource::ResponseJson => quote! { #ident: deserialized.#ident.into() },
        }
    })
}

struct ChainOutputFieldSpec<'a> {
    field: &'a syn::Field,
    output_source: ChainOutputSource,
    rename: Option<LitStr>,
}

impl<'a> ChainOutputFieldSpec<'a> {
    fn new(
        field: &'a syn::Field,
        langchain_attrs: LangchainFieldAttrs,
        serde_attrs: SerdeFieldAttrs,
    ) -> Self {
        Self {
            field,
            output_source: langchain_attrs.output_source.unwrap_or_default(),
            rename: serde_attrs.rename,
        }
    }
}

struct ChainOutputStructSpec {
    rename_all: Option<RenameAll>,
    from_input: Option<syn::Type>,
    crate_path: syn::Path,
    serde_path: syn::Path,
    serde_json_path: syn::Path,
}

impl ChainOutputStructSpec {
    fn new(langchain_attrs: LangchainStructAttrs, serde_attrs: SerdeStructAttrs) -> Self {
        let crate_path = langchain_attrs
            .crate_path
            .unwrap_or_else(default_crate_path);
        let serde_path = serde_attrs
            .crate_path
            .or(langchain_attrs.serde_path)
            .unwrap_or_else(default_serde_path);
        let serde_json_path = langchain_attrs
            .serde_json_path
            .unwrap_or_else(default_serde_json_path);

        Self {
            rename_all: serde_attrs.rename_all,
            from_input: langchain_attrs.from_input,
            crate_path,
            serde_path,
            serde_json_path,
        }
    }
}

pub fn derive_chain_output(
    input: syn::DeriveInput,
) -> Result<proc_macro2::TokenStream, proc_macro_error::Diagnostic> {
    let struct_name = &input.ident;
    let fields = &get_fields(&input.data)?.named;

    let ChainOutputStructSpec {
        rename_all,
        from_input,
        crate_path,
        serde_path,
        serde_json_path,
    } = ChainOutputStructSpec::new(
        extract_attr(&input.attrs, get_chain_struct_attrs)?,
        extract_attr(&input.attrs, get_serde_struct_attrs)?,
    );
    let field_specs = fields
        .into_iter()
        .map(|field| -> Result<_, Diagnostic> {
            let langchain_attrs = extract_attr(&field.attrs, get_langchain_field_attrs)?;
            let serde_attrs = extract_attr(&field.attrs, get_serde_field_attrs)?;

            Ok(ChainOutputFieldSpec::new(
                field,
                langchain_attrs,
                serde_attrs,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    if let Some(f) = {
        field_specs
            .iter()
            .filter(|f| f.output_source == ChainOutputSource::Response)
            .nth(1)
    } {
        return Err(Diagnostic::spanned(
            f.field.span(),
            Level::Error,
            "ChainOutput struct cannot have more than one field with `#[langchain(from = \"response\")]` attribute".into(),
        ));
    }

    if from_input.is_none()
        && field_specs
            .iter()
            .any(|f| f.output_source == ChainOutputSource::Input)
    {
        return Err(Diagnostic::new(
            Level::Error,
            "ChainOutput struct must have a `#[langchain(from_input = \"...\")]` attribute when it has fields with `#[langchain(from = \"input\")]`".into(),
        ));
    }

    let mut generics_with_in = input.generics.clone();
    let ((impl_generics, ty_generics, where_clause), co_generic) = match from_input.clone() {
        Some(co_generic) => (input.generics.split_for_impl(), co_generic),
        None => {
            generics_with_in.params.push(syn::parse_quote!(IN));
            let (_, ty_generics, _) = input.generics.split_for_impl();
            let (impl_generics, _, where_clause) = generics_with_in.split_for_impl();
            (
                (impl_generics, ty_generics, where_clause),
                syn::parse_quote!(IN),
            )
        }
    };

    let deserialized = field_specs
        .iter()
        .any(|f| f.output_source == ChainOutputSource::ResponseJson)
        .then(|| {
            let deser_struct = deser_struct(&field_specs, &serde_path, &rename_all);
            quote! {
                #deser_struct
                let value = #crate_path::output_parser::parse_partial_json(&original, false)?;
                let deserialized: InputDeserialize = #serde_json_path::from_value(value)?;
            }
        });
    let field_initializers = field_initializers(&field_specs, &rename_all);

    let fn_body = quote! {
        let original = text.into();
        #deserialized
        Ok(Self {
            #(#field_initializers),*
        })
    };

    let fn_signature = if field_specs
        .iter()
        .any(|f| f.output_source == ChainOutputSource::Input)
    {
        quote! { fn construct_from_text_and_input(input: #from_input, text: impl Into<String>) -> Result<Self, #crate_path::output_parser::OutputParseError> }
    } else {
        quote! { fn construct_from_text(text: impl Into<String>) -> Result<Self, #crate_path::output_parser::OutputParseError> }
    };

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::chain::ChainOutput<#co_generic> for #struct_name #ty_generics
        #where_clause
        {
            #fn_signature {
                #fn_body
            }
        }
    };

    Ok(expanded)
}
