use quote::{format_ident, quote};

use crate::{
    attr::{ChainOutputFieldAttr, SerdeFieldAttrs},
    rename::{RenameAll, get_renamed_key},
};

pub fn deser_struct(
    fields_with_attrs: &[(&syn::Field, ChainOutputFieldAttr, SerdeFieldAttrs)],
    serde_path: &syn::Path,
    rename_all: &Option<RenameAll>,
) -> proc_macro2::TokenStream {
    let deser_fields = fields_with_attrs
        .iter()
        .filter(|(_, attr, _)| *attr == ChainOutputFieldAttr::ResponseJson)
        .map(|(field, _, serde_attrs)| {
            let ident = field.ident.as_ref().unwrap();
            let rename_attr = serde_attrs.rename.as_ref().map(|r| {
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
        pub struct InputDeserialize {
            #(#deser_fields),*
        }
    }
}

pub fn field_initializers(
    fields_with_attrs: &[(&syn::Field, ChainOutputFieldAttr, SerdeFieldAttrs)],
    rename_all: &Option<RenameAll>,
) -> impl Iterator<Item = proc_macro2::TokenStream> {
    fields_with_attrs.iter().map(|(field, attr, serde_attrs)| {
        let ident = field.ident.as_ref().unwrap();
        let key = get_renamed_key(field, &serde_attrs.rename, rename_all);

        match attr {
            ChainOutputFieldAttr::Input => {
                let key_ident = format_ident!("{key}");
                quote! { #ident: input.#key_ident.into() }
            }
            ChainOutputFieldAttr::Response => quote! { #ident: original.into() },
            ChainOutputFieldAttr::ResponseJson => quote! { #ident: deserialized.#ident.into() },
        }
    })
}
