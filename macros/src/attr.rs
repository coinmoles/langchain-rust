use std::str::FromStr;

use proc_macro_error::{Diagnostic, Level};
use syn::{Attribute, LitStr, Path, parse_str, spanned::Spanned};

use crate::rename::RenameAll;

#[derive(Default)]
pub struct SerdeFieldAttrs {
    pub rename: Option<LitStr>,
}

#[derive(Default)]
pub struct LangchainFieldAttrs {
    pub output_source: Option<ChainOutputSource>,
    pub input_kind: Option<ChainInputKind>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChainOutputSource {
    Input,
    Response,
    #[default]
    ResponseJson,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChainInputKind {
    Inner,
    #[default]
    Text,
    Placeholder,
}

#[derive(Default)]
pub struct SerdeStructAttrs {
    pub rename_all: Option<RenameAll>,
    pub crate_path: Option<syn::Path>,
}

#[derive(Default)]
pub struct LangchainStructAttrs {
    pub from_input: Option<syn::Type>,
    pub crate_path: Option<syn::Path>,
    pub serde_path: Option<syn::Path>,
    pub serde_json_path: Option<syn::Path>,
}

pub fn extract_attr<T: Default>(
    attrs: &[Attribute],
    get_attr: impl Fn(&Attribute) -> Result<Option<T>, Diagnostic>,
) -> Result<T, Diagnostic> {
    attrs
        .iter()
        .find_map(|attr| get_attr(attr).transpose())
        .unwrap_or_else(|| Ok(T::default()))
}

pub fn get_langchain_field_attrs(
    attr: &Attribute,
) -> Result<Option<LangchainFieldAttrs>, Diagnostic> {
    if !attr.path().is_ident("langchain") {
        return Ok(None);
    }

    let mut from = None;
    let mut into = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            from = match lit.value().as_str() {
                "input" => Some(ChainOutputSource::Input),
                "response" => Some(ChainOutputSource::Response),
                "response_json" => Some(ChainOutputSource::ResponseJson),
                _ => return Err(syn::Error::new_spanned(
                    lit,
                    "Invalid value for `#[langchain(from = ...)]`, expected `input`, `response`, or `response_json`",
                )),
            };
        } else if meta.path.is_ident("into") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            into = match lit.value().as_str() {
                "inner" => Some(ChainInputKind::Inner),
                "text" => Some(ChainInputKind::Text),
                "placeholder" => Some(ChainInputKind::Placeholder),
                _ => return Err(syn::Error::new_spanned(
                    lit,
                    "Invalid value for `#[langchain(into = ...)]`, expected `inner`, `text`, or `placeholder`",
                )),
            };
        }else {
            return Err(syn::Error::new_spanned(
                meta.path,
                "Unknown key in `#[langchain(...)]`",
            ));
        }

        Ok(())
    })
    .map_err(|e|
        Diagnostic::spanned(
            e.span(),
            Level::Error,
            format!("Failed to parse langchain attribute: {e}"
        )
    ))?;

    Ok(Some(LangchainFieldAttrs {
        output_source: from,
        input_kind: into,
    }))
}

pub fn get_serde_field_attrs(attr: &Attribute) -> Result<Option<SerdeFieldAttrs>, Diagnostic> {
    if !attr.path().is_ident("serde") {
        return Ok(None);
    }

    let mut rename = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            rename = Some(lit);
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

    Ok(Some(SerdeFieldAttrs { rename }))
}

pub fn get_chain_struct_attrs(
    attr: &Attribute,
) -> Result<Option<LangchainStructAttrs>, Diagnostic> {
    if !attr.path().is_ident("langchain") {
        return Ok(None);
    }
    let mut from_input = None;
    let mut crate_path: Option<Path> = None;
    let mut serde_path: Option<Path> = None;
    let mut serde_json_path: Option<Path> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("crate") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            crate_path = Some(parse_str(&lit.value()).expect("Invalid crate path"));
        } else if meta.path.is_ident("serde") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            serde_path = Some(parse_str(&lit.value()).expect("Invalid serde path"));
        } else if meta.path.is_ident("serde_json") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            serde_json_path = Some(parse_str(&lit.value()).expect("Invalid serde_json path"));
        } else if meta.path.is_ident("from_input") {
            let value = meta.value()?;
            from_input = Some(value.parse()?);
        } else {
            return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
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

    Ok(Some(LangchainStructAttrs {
        from_input,
        crate_path,
        serde_path,
        serde_json_path,
    }))
}

pub fn get_serde_struct_attrs(attr: &Attribute) -> Result<Option<SerdeStructAttrs>, Diagnostic> {
    if !attr.path().is_ident("serde") {
        return Ok(None);
    }
    let mut rename_all = None;
    let mut crate_path = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename_all") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            let v = lit.value();
            rename_all =
                Some(RenameAll::from_str(&v).map_err(|e| syn::Error::new_spanned(lit, e))?);
        }
        if meta.path.is_ident("crate") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            crate_path = Some(parse_str(&lit.value()).expect("Invalid crate path"));
        } else {
            return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
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

    Ok(Some(SerdeStructAttrs {
        rename_all,
        crate_path,
    }))
}
