use proc_macro_error::{Diagnostic, Level};
use syn::{Attribute, Ident, LitStr, Meta, spanned::Spanned};

use crate::rename::RenameAll;

pub struct FieldAttrs {
    pub input: ChainInputFieldAttr,
    pub serde_rename: Option<String>,
}

pub struct StructAttrs {
    pub serde_rename_all: Option<RenameAll>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainInputFieldAttr {
    Inner,
    Text,
    Placeholder,
}

pub fn extract_field_attrs(attrs: &[Attribute]) -> Result<Option<FieldAttrs>, Diagnostic> {
    let Some(input) = attrs
        .iter()
        .find_map(|attr| get_chain_input_field_attrs(attr).transpose())
        .transpose()?
    else {
        return Ok(None);
    };
    let serde_rename = attrs
        .iter()
        .find_map(|attr| get_serde_field_attrs(attr).transpose())
        .transpose()?;

    Ok(Some(FieldAttrs {
        input,
        serde_rename,
    }))
}

pub fn extract_struct_attrs(attrs: &[Attribute]) -> Result<StructAttrs, Diagnostic> {
    let serde_rename_all = attrs
        .iter()
        .find_map(|attr| get_serde_struct_attrs(attr).transpose())
        .transpose()?;

    Ok(StructAttrs { serde_rename_all })
}

fn get_chain_input_field_attrs(
    attr: &Attribute,
) -> Result<Option<ChainInputFieldAttr>, Diagnostic> {
    if !attr.path().is_ident("chain_input") {
        return Ok(None);
    }
    let Meta::List(meta_list) = &attr.meta else {
        return Err(Diagnostic::spanned(
            attr.span(),
            Level::Error,
            "`#[chain_input(...)]` must use list syntax, e.g., `#[chain_input(text)]`".into(),
        ));
    };
    let Ok(ident) = meta_list.parse_args::<Ident>() else {
        return Err(Diagnostic::spanned(
            meta_list.span(),
            Level::Error,
            "`#[chain_input(...)]` must contain a single identifier: `text`, `inner`, or `placeholder`"
                .into(),
        ));
    };
    match ident.to_string().as_str() {
        "inner" => Ok(Some(ChainInputFieldAttr::Inner)),
        "text" => Ok(Some(ChainInputFieldAttr::Text)),
        "placeholder" => Ok(Some(ChainInputFieldAttr::Placeholder)),
        _ => Err(Diagnostic::spanned(
            ident.span(),
            Level::Error,
            "Invalid value for `#[chain_input(...)]`, expected `inner`, `text`, or `placeholder`"
                .into(),
        )),
    }
}

fn get_serde_field_attrs(attr: &Attribute) -> Result<Option<String>, Diagnostic> {
    if !attr.path().is_ident("serde") {
        return Ok(None);
    }

    let mut serde_rename: Option<String> = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            serde_rename = Some(lit.value());
        }

        Ok(())
    })
    .map_err(|e| {
        Diagnostic::spanned(
            e.span(),
            Level::Error,
            format!("Failed to parse serde attribute: {}", e),
        )
    })?;

    Ok(serde_rename)
}

fn get_serde_struct_attrs(attr: &Attribute) -> Result<Option<RenameAll>, Diagnostic> {
    if !attr.path().is_ident("serde") {
        return Ok(None);
    }
    let mut rename_all = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename_all") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            let v = lit.value();
            rename_all = Some(RenameAll::from_str(&v).ok_or_else(|| {
                syn::Error::new_spanned(lit, format!("Invalid rename_all value: {v}"))
            })?);
        }
        Ok(())
    })
    .map_err(|e| {
        Diagnostic::spanned(
            e.span(),
            Level::Error,
            format!("Failed to parse serde attribute: {}", e),
        )
    })?;

    Ok(rename_all)
}
