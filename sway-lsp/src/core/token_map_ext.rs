//! This module provides the `TokenMapExt` trait, which extends iterators over tokens with
//! additional functionality, such as finding all references of a given token in a TokenMap.
//!
//! The `TokenMapExt` trait is implemented for any iterator that yields (Ident, Token) pairs.

use crate::core::token::{LspSpan, Token};
use sway_core::Engines;

/// A trait for extending iterators with the `all_references_of_token` method.
pub trait TokenMapExt: Sized {
    /// Find all references in the TokenMap for a given token.
    ///
    /// This is useful for the highlighting and renaming LSP capabilities.
    fn all_references_of_token<'s>(
        self,
        token_to_match: &'s Token,
        engines: &'s Engines,
    ) -> AllReferencesOfToken<'s, Self>;
}

/// Implement `TokenMapExt` for any iterator that yields (Ident, Token) pairs.
impl<I> TokenMapExt for I
where
    I: Iterator<Item = (LspSpan, Token)>,
{
    fn all_references_of_token<'s>(
        self,
        token_to_match: &'s Token,
        engines: &'s Engines,
    ) -> AllReferencesOfToken<'s, Self> {
        AllReferencesOfToken {
            token_to_match,
            engines,
            iter: self,
        }
    }
}

/// A custom iterator that returns all references of a given token.
pub struct AllReferencesOfToken<'s, I> {
    token_to_match: &'s Token,
    engines: &'s Engines,
    iter: I,
}

impl<'s, I> Iterator for AllReferencesOfToken<'s, I>
where
    I: Iterator<Item = (LspSpan, Token)>,
{
    type Item = (LspSpan, Token);

    fn next(&mut self) -> Option<Self::Item> {
        for (lsp_span, token) in self.iter.by_ref() {
            let decl_lsp_span_to_match = self.token_to_match.declared_token_lsp_span(self.engines);
            let is_same_type =
                decl_lsp_span_to_match == token.declared_token_lsp_span(self.engines);
            let is_decl_of_token = Some(&lsp_span) == decl_lsp_span_to_match.as_ref();

            if decl_lsp_span_to_match.is_some() && is_same_type || is_decl_of_token {
                return Some((lsp_span, token));
            }
        }
        None
    }
}
