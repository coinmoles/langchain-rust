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
