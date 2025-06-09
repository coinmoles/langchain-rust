use proc_macro_error::{Diagnostic, Level};
use syn::{Attribute, Ident, LitStr, Meta, Path, parse_str, spanned::Spanned};

use crate::rename::RenameAll;

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

#[derive(Default)]
pub struct SerdeStructAttrs {
    pub rename_all: Option<RenameAll>,
}

#[derive(Default)]
pub struct ChainInputStructAttr {
    pub crate_path: Option<syn::Path>,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ChainOutputStructAttr {
//     Single,
//     Deserialize,
// }

pub fn extract_attr<T>(
    attrs: &[Attribute],
    get_attr: impl Fn(&Attribute) -> Result<Option<T>, Diagnostic>,
) -> Result<Option<T>, Diagnostic> {
    attrs
        .iter()
        .find_map(|attr| get_attr(attr).transpose())
        .transpose()
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

pub fn get_chain_input_struct_attrs(
    attr: &Attribute,
) -> Result<Option<ChainInputStructAttr>, Diagnostic> {
    if !attr.path().is_ident("chain") {
        return Ok(None);
    }
    let mut crate_path: Option<Path> = None;

    attr.parse_nested_meta(|meta| {
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

    Ok(Some(ChainInputStructAttr { crate_path }))
}

// pub fn get_chain_output_struct_attrs(
//     attr: &Attribute,
// ) -> Result<Option<ChainInputStructAttr>, Diagnostic> {
//     if !attr.path().is_ident("chain_output") {
//         return Ok(None);
//     }
//     let mut crate_path: Option<Path> = None;

//     attr.parse_nested_meta(|meta| {
//         if meta.path.is_ident("crate") {
//             let value = meta.value()?;
//             let lit: LitStr = value.parse()?;
//             crate_path = Some(parse_str(&lit.value()).expect("Invalid crate path"));
//         } else {
//             return Err(syn::Error::new(meta.path.span(), "Unknown attribute"));
//         }

//         Ok(())
//     })
//     .map_err(|e| {
//         Diagnostic::spanned(
//             e.span(),
//             Level::Error,
//             format!("Failed to parse serde attribute: {}", e),
//         )
//     })?;

//     Ok(Some(ChainInputStructAttr { crate_path }))
// }

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

    Ok(Some(SerdeStructAttrs { rename_all }))
}
