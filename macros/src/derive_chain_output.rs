use proc_macro_error::{Diagnostic, Level};
use quote::{format_ident, quote};
use syn::{LitStr, spanned::Spanned};

use crate::{
    attr::{
        ChainOutputSource, LangchainFieldAttrs, LangchainStructAttrs, SerdeFieldAttrs,
        SerdeStructAttrs, extract_attr, get_chain_struct_attrs, get_langchain_field_attrs,
        get_serde_field_attrs, get_serde_struct_attrs,
    },
    crate_path::{default_crate_path, default_serde_json_path, default_serde_path},
    helpers::{get_fields, get_renamed_key},
    rename::RenameAll,
};

fn deser_struct(
    field_specs: &[ChainOutputFieldSpec<'_>],
    serde_path: &syn::Path,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let deser_fields = field_specs
        .iter()
        .filter(|f| f.output_source == ChainOutputSource::ResponseJson)
        .map(|f| {
            let ident = f.field.ident.as_ref().unwrap();
            let rename_attr = f.rename.as_ref().map(|r| {
                quote! {
                    #[serde(rename = #r)]
                }
            });

            quote! {
                #rename_attr
                pub #ident: String
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
    let fields = &get_fields(&input)?.named;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

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

    let Some(from_input) = from_input else {
        return Err(Diagnostic::new(
            Level::Error,
            "ChainOutput struct must have a `#[langchain(from_input = \"...\")]` attribute".into(),
        ));
    };

    let deser_struct = deser_struct(&field_specs, &serde_path, &rename_all);
    let field_initializers = field_initializers(&field_specs, &rename_all);

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #crate_path::schemas::ChainOutput<#from_input> for #struct_name #ty_generics
        #where_clause
        {
            fn parse_output(input: #from_input, output: impl Into<String>) -> Result<Self, #crate_path::schemas::OutputParseError> {
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

    Ok(expanded)
}
