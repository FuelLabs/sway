use crate::core::token::Token;
use dashmap::DashMap;
use sway_core::TypeEngine;
use sway_types::{Ident, Span};
use tower_lsp::lsp_types::Url;

#[derive(Debug)]
pub struct TokenMap(DashMap<(Ident, Span), Token>);

impl TokenMap {
    pub fn new() -> TokenMap {
        TokenMap(DashMap::new())
    }

    /// Return a Iterator for tokens belonging to the provided file path
    pub fn tokens_for_file<'s>(
        &'s self,
        uri: &'s Url,
    ) -> impl 's + Iterator<Item = (Ident, Token)> {
        self.iter()
            .filter(|item| {
                let (_, span) = item.key();
                match span.path() {
                    Some(path) => path.to_str() == Some(uri.path()),
                    None => false,
                }
            })
            .map(|item| {
                let ((ident, _), token) = item.pair();
                (ident.clone(), token.clone())
            })
    }

    /// Find all references in the TokenMap for a given token.
    ///
    /// This is useful for the highlighting and renaming LSP capabilities.
    pub fn all_references_of_token<'s>(
        &'s self,
        token: &Token,
        type_engine: &'s TypeEngine,
    ) -> impl 's + Iterator<Item = (Ident, Token)> {
        let current_type_id = token.declared_token_span(type_engine);

        self.iter()
            .filter(move |item| {
                let ((_, _), token) = item.pair();
                current_type_id == token.declared_token_span(type_engine)
            })
            .map(|item| {
                let ((ident, _), token) = item.pair();
                (ident.clone(), token.clone())
            })
    }
}

impl std::ops::Deref for TokenMap {
    type Target = DashMap<(Ident, Span), Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
