use proc_macro_crate::{FoundCrate, crate_name};
use syn::parse_str;

pub fn default_crate_path() -> syn::Path {
    match (
        crate_name("langchain-rust"),
        std::env::var("CARGO_CRATE_NAME").as_deref(),
    ) {
        (Ok(FoundCrate::Itself), Ok("langchain_rust")) => parse_str("crate").unwrap(),
        (Ok(FoundCrate::Name(name)), _) => parse_str(&name).unwrap(),
        _ => parse_str("::langchain_rust").unwrap(),
    }
}

pub fn default_serde_path() -> syn::Path {
    match (
        crate_name("serde"),
        std::env::var("CARGO_CRATE_NAME").as_deref(),
    ) {
        (Ok(FoundCrate::Itself), Ok("serde")) => parse_str("crate").unwrap(),
        (Ok(FoundCrate::Name(name)), _) => parse_str(&name).unwrap(),
        _ => parse_str("::serde").unwrap(),
    }
}

// pub fn default_serde_json_path() -> syn::Path {
//     match (
//         crate_name("serde_json"),
//         std::env::var("CARGO_CRATE_NAME").as_deref(),
//     ) {
//         (Ok(FoundCrate::Itself), Ok("serde_json")) => parse_str("crate").unwrap(),
//         (Ok(FoundCrate::Name(name)), _) => parse_str(&name).unwrap(),
//         _ => parse_str("::serde_json").unwrap(),
//     }
// }
