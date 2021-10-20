use proc_macro2::{Ident, Span};
use syn::Ident as SynIdent;

/// Expands a identifier string into an token.
pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

// Expands an identifier string into a token and appending `_` if the
/// identifier is for a reserved keyword.
///
/// Parsing keywords like `self` can fail, in this case we add an underscore.
pub fn safe_ident(name: &str) -> Ident {
    syn::parse_str::<SynIdent>(name).unwrap_or_else(|_| ident(&format!("{}_", name)))
}
