use std::str::FromStr;

use proc_macro_error::{Diagnostic, Level};
use syn::{Attribute, Ident, LitStr, Meta, Path, parse_str, spanned::Spanned};

use crate::{
    crate_path::{default_crate_path, default_serde_json_path, default_serde_path},
    rename::RenameAll,
};

#[derive(Default)]
pub struct SerdeFieldAttrs {
    pub rename: Option<LitStr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainInputFieldAttr {
    Inner,
    Text,
    Placeholder,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ChainOutputFieldAttr {
    Input,
    Response,
    #[default]
    ResponseJson,
}

#[derive(Default)]
pub struct SerdeStructAttrs {
    pub rename_all: Option<RenameAll>,
}

pub struct ChainStructAttr {
    pub crate_path: syn::Path,
    pub serde_path: syn::Path,
    pub serde_json_path: syn::Path,
}

impl Default for ChainStructAttr {
    fn default() -> Self {
        Self {
            crate_path: default_crate_path(),
            serde_path: default_serde_path(),
            serde_json_path: default_serde_json_path(),
        }
    }
}

#[derive(Default)]
pub struct ChainOutputStructAttrs {
    pub input: Option<syn::Type>,
}

pub fn extract_attr<T>(
    attrs: &[Attribute],
    get_attr: impl Fn(&Attribute) -> Result<Option<T>, Diagnostic>,
) -> Result<Option<T>, Diagnostic> {
    attrs
        .iter()
        .find_map(|attr| get_attr(attr).transpose())
        .transpose()
}

pub fn extract_attr_default<T: Default>(
    attrs: &[Attribute],
    get_attr: impl Fn(&Attribute) -> Result<Option<T>, Diagnostic>,
) -> Result<T, Diagnostic> {
    extract_attr(attrs, get_attr).map(|opt| opt.unwrap_or_default())
}

pub fn get_chain_input_field_attr(
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

pub fn get_chain_output_field_attr(
    attr: &Attribute,
) -> Result<Option<ChainOutputFieldAttr>, Diagnostic> {
    if !attr.path().is_ident("chain_output") {
        return Ok(None);
    }

    let Meta::List(meta_list) = &attr.meta else {
        return Err(Diagnostic::spanned(
            attr.span(),
            Level::Error,
            "`#[chain_output(...)]` must use list syntax, e.g., `#[chain_output(from_input)]`"
                .into(),
        ));
    };
    let Ok(ident) = meta_list.parse_args::<Ident>() else {
        return Err(Diagnostic::spanned(
            meta_list.span(),
            Level::Error,
            "`#[chain_output(...)]` must contain a single identifier: `from_input`, `from_response`, or `from_response_json`"
                .into(),
        ));
    };
    match ident.to_string().as_str() {
        "from_input" => Ok(Some(ChainOutputFieldAttr::Input)),
        "from_response" => Ok(Some(ChainOutputFieldAttr::Response)),
        "from_response_json" => Ok(Some(ChainOutputFieldAttr::ResponseJson)),
        _ => Err(Diagnostic::spanned(
            ident.span(),
            Level::Error,
            "Invalid value for `#[chain_output(...)]`, expected `from_input`, `from_response`, or `from_response_json`"
                .into(),
        )),
    }
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

pub fn get_chain_struct_attrs(attr: &Attribute) -> Result<Option<ChainStructAttr>, Diagnostic> {
    if !attr.path().is_ident("chain") {
        return Ok(None);
    }
    let mut crate_path: Option<Path> = None;
    let mut serde_path: Option<Path> = None;
    let mut serde_json_path: Option<Path> = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("crate") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            crate_path = Some(parse_str(&lit.value()).expect("Invalid crate path"));
        } else {
            return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
        }
        if meta.path.is_ident("serde") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            serde_path = Some(parse_str(&lit.value()).expect("Invalid serde path"));
        } else if meta.path.is_ident("serde_json") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            serde_json_path = Some(parse_str(&lit.value()).expect("Invalid serde_json path"));
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

    Ok(Some(ChainStructAttr {
        crate_path: crate_path.unwrap_or_else(default_crate_path),
        serde_path: serde_path.unwrap_or_else(default_serde_path),
        serde_json_path: serde_json_path.unwrap_or_else(default_serde_json_path),
    }))
}

pub fn get_chain_output_struct_attrs(
    attr: &Attribute,
) -> Result<Option<ChainOutputStructAttrs>, Diagnostic> {
    if !attr.path().is_ident("chain_output") {
        return Ok(None);
    }
    let mut input = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("input") {
            let value = meta.value()?;
            let lit = value.parse()?;
            input = Some(lit);
        } else {
            return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
        }

        Ok(())
    })
    .map_err(|e| {
        Diagnostic::spanned(
            e.span(),
            Level::Error,
            format!("Failed to parse chain_output attribute: {}", e),
        )
    })?;

    Ok(Some(ChainOutputStructAttrs { input }))
}

pub fn get_serde_struct_attrs(attr: &Attribute) -> Result<Option<SerdeStructAttrs>, Diagnostic> {
    if !attr.path().is_ident("serde") {
        return Ok(None);
    }
    let mut rename_all = None;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename_all") {
            let value = meta.value()?;
            let lit: LitStr = value.parse()?;
            let v = lit.value();
            rename_all =
                Some(RenameAll::from_str(&v).map_err(|e| syn::Error::new_spanned(lit, e))?);
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

    Ok(Some(SerdeStructAttrs { rename_all }))
}
