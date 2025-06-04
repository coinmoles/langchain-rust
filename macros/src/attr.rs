use proc_macro_crate::{FoundCrate, crate_name};
use syn::{Attribute, Expr, Ident, Lit, Meta, Path, parse_str};

use crate::rename::RenameAll;

pub fn is_text_input_attr(attr: &Attribute) -> bool {
    if !attr.path().is_ident("input") {
        return false;
    }
    let Meta::List(meta_list) = &attr.meta else {
        return false;
    };
    let Ok(ident) = meta_list.parse_args::<Ident>() else {
        return false;
    };

    ident == "text"
}

pub fn is_placeholder_input_attr(attr: &Attribute) -> bool {
    if !attr.path().is_ident("input") {
        return false;
    }
    let Meta::List(meta_list) = &attr.meta else {
        return false;
    };
    let Ok(ident) = meta_list.parse_args::<Ident>() else {
        return false;
    };

    ident == "placeholder"
}

fn get_serde_rename_all(attr: &Attribute) -> Option<RenameAll> {
    if !attr.path().is_ident("serde") {
        return None;
    }
    let Meta::NameValue(meta_name_value) = &attr.meta else {
        return None;
    };
    if !meta_name_value.path.is_ident("rename_all") {
        return None;
    }
    let Expr::Lit(lit) = &meta_name_value.value else {
        return None;
    };
    let Lit::Str(lit_str) = &lit.lit else {
        return None;
    };
    RenameAll::from_str(&lit_str.value())
}

pub fn extract_serde_rename_all(attrs: &[Attribute]) -> Option<RenameAll> {
    attrs.iter().find_map(get_serde_rename_all)
}

fn get_serde_rename(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("serde") {
        return None;
    }
    let Meta::NameValue(meta_name_value) = &attr.meta else {
        return None;
    };
    if !meta_name_value.path.is_ident("rename") {
        return None;
    }
    let Expr::Lit(lit) = &meta_name_value.value else {
        return None;
    };
    let Lit::Str(lit_str) = &lit.lit else {
        return None;
    };
    Some(lit_str.value())
}

pub fn extract_serde_rename(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(get_serde_rename)
}

fn get_crate_path(attr: &Attribute) -> Option<syn::Path> {
    if !attr.path().is_ident("input") {
        return None;
    }
    let Meta::NameValue(meta_name_value) = &attr.meta else {
        return None;
    };
    if !meta_name_value.path.is_ident("crate") {
        return None;
    }
    let Expr::Lit(lit) = &meta_name_value.value else {
        return None;
    };
    let Lit::Str(lit_str) = &lit.lit else {
        return None;
    };
    Some(lit_str.parse::<Path>().expect("Invalid path"))
}

pub fn extract_crate_path(attrs: &[Attribute]) -> Path {
    attrs.iter().find_map(get_crate_path).unwrap_or_else(|| {
        match (
            crate_name("langchain-rust"),
            std::env::var("CARGO_CRATE_NAME").as_deref(),
        ) {
            (Ok(FoundCrate::Itself), Ok("langchain_rust")) => parse_str("crate").unwrap(),
            (Ok(FoundCrate::Name(name)), _) => parse_str(&name).unwrap(),
            _ => parse_str("::langchain_rust").unwrap(),
        }
    })
}
