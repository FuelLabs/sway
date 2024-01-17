use crate::core::token::{self, Token, TokenIdent, TypedAstToken};
use dashmap::{mapref::one::RefMut, try_result::TryResult, DashMap};
use lsp_types::{Position, Url};
use std::{thread, time::Duration};
use sway_core::{language::ty, type_system::TypeId, Engines};
use sway_types::{Ident, SourceEngine, Spanned};

// Re-export the TokenMapExt trait.
pub use crate::core::token_map_ext::TokenMapExt;

/// The TokenMap is the main data structure of the language server.
/// It stores all of the tokens that have been parsed and typechecked by the sway compiler.
///
/// The TokenMap is a wrapper around a [DashMap], which is a concurrent HashMap.
#[derive(Debug, Default)]
pub struct TokenMap(DashMap<TokenIdent, Token>);

impl<'a> TokenMap {
    /// Create a new token map.
    pub fn new() -> TokenMap {
        TokenMap(DashMap::with_capacity(2048))
    }

    /// Attempts to get a mutable reference to a token with retries on lock.
    /// Retries up to 8 times with increasing backoff (1ns, 10ns, 100ns, 500ns, 1µs, 10µs, 100µs, 1ms).
    pub fn try_get_mut_with_retry(
        &'a self,
        ident: &TokenIdent,
    ) -> Option<RefMut<TokenIdent, Token>> {
        const MAX_RETRIES: usize = 8;
        let backoff_times = [1, 10, 100, 500, 1_000, 10_000, 100_000, 1_000_000]; // Backoff times in nanoseconds
        for (i, sleep) in backoff_times.iter().enumerate().take(MAX_RETRIES) {
            match self.try_get_mut(ident) {
                TryResult::Present(token) => return Some(token),
                TryResult::Absent => return None,
                TryResult::Locked => {
                    tracing::warn!(
                        "Failed to get token, retrying attmpt {}: {:#?}",
                        i,
                        ident.name
                    );
                    // Wait for the specified backoff time before retrying
                    let backoff_time = Duration::from_nanos(*sleep);
                    thread::sleep(backoff_time);
                }
            }
        }
        tracing::error!(
            "Failed to get token after {} retries: {:#?}",
            MAX_RETRIES,
            ident
        );
        None // Return None if all retries are exhausted
    }

    /// Create a custom iterator for the TokenMap.
    ///
    /// The iterator returns ([Ident], [Token]) pairs.
    pub fn iter(&self) -> TokenMapIter {
        TokenMapIter {
            inner_iter: self.0.iter(),
        }
    }

    /// Return an Iterator of tokens belonging to the provided [Url].
    pub fn tokens_for_file<'s>(
        &'s self,
        uri: &'s Url,
    ) -> impl 's + Iterator<Item = (TokenIdent, Token)> {
        self.iter().flat_map(|(ident, token)| {
            ident.path.as_ref().and_then(|path| {
                if path.to_str() == Some(uri.path()) {
                    Some((ident.clone(), token.clone()))
                } else {
                    None
                }
            })
        })
    }

    /// Return an Iterator of tokens matching the given name.
    pub fn tokens_for_name<'s>(
        &'s self,
        name: &'s String,
    ) -> impl 's + Iterator<Item = (TokenIdent, Token)> {
        self.iter().flat_map(|(ident, token)| {
            if ident.name == *name {
                Some((ident.clone(), token.clone()))
            } else {
                None
            }
        })
    }

    /// Given a cursor [Position], return the [TokenIdent] of a token in the
    /// Iterator if one exists at that position.
    pub fn idents_at_position<I>(&self, cursor_position: Position, tokens: I) -> Vec<TokenIdent>
    where
        I: Iterator<Item = (TokenIdent, Token)>,
    {
        tokens
            .filter_map(|(ident, _)| {
                if cursor_position >= ident.range.start && cursor_position <= ident.range.end {
                    return Some(ident);
                }
                None
            })
            .collect()
    }

    /// Returns the first parent declaration found at the given cursor position.
    ///
    /// For example, if the cursor is inside a function body, this function returns the function declaration.
    pub fn parent_decl_at_position(
        &self,
        source_engine: &SourceEngine,
        uri: &Url,
        position: Position,
    ) -> Option<(TokenIdent, Token)> {
        self.tokens_at_position(source_engine, uri, position, None)
            .iter()
            .find_map(|(ident, token)| {
                if let Some(TypedAstToken::TypedDeclaration(_)) = token.typed {
                    Some((ident.clone(), token.clone()))
                } else {
                    None
                }
            })
    }

    /// Returns the first collected tokens that is at the cursor position.
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(TokenIdent, Token)> {
        let tokens = self.tokens_for_file(uri);
        self.idents_at_position(position, tokens)
            .first()
            .and_then(|ident| {
                self.try_get(ident).try_unwrap().map(|item| {
                    let (ident, token) = item.pair();
                    (ident.clone(), token.clone())
                })
            })
    }

    /// Returns all collected tokens that are at the given [Position] in the file.
    /// If `functions_only` is true, it only returns tokens of type [TypedAstToken::TypedFunctionDeclaration].
    ///
    /// This is different from `idents_at_position` because this searches the spans of token bodies, not
    /// just the spans of the token idents. For example, if we want to find out what function declaration
    /// the cursor is inside of, we need to search the body of the function declaration, not just the ident
    /// of the function declaration (the function name).
    pub fn tokens_at_position(
        &self,
        source_engine: &SourceEngine,
        uri: &Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<(TokenIdent, Token)> {
        self.tokens_for_file(uri)
            .filter_map(|(ident, token)| {
                let token_ident = match token.typed {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl))
                        if functions_only == Some(true) =>
                    {
                        TokenIdent::new(&Ident::new(decl.span), source_engine)
                    }
                    Some(TypedAstToken::TypedDeclaration(decl)) => {
                        TokenIdent::new(&Ident::new(decl.span()), source_engine)
                    }
                    // Seems to be a clippy bug
                    #[allow(clippy::redundant_clone)]
                    _ => ident.clone(),
                };
                if position >= token_ident.range.start && position <= token_ident.range.end {
                    return self.try_get(&ident).try_unwrap().map(|item| {
                        let (ident, token) = item.pair();
                        (ident.clone(), token.clone())
                    });
                }
                None
            })
            .filter_map(|(ident, token)| {
                if functions_only == Some(true) {
                    if let Some(TypedAstToken::TypedFunctionDeclaration(_)) = token.typed {
                        return Some((ident, token));
                    }
                    return None;
                }
                Some((ident, token))
            })
            .collect()
    }

    /// Uses the [TypeId] to find the associated [ty::TyDecl] in the TokenMap.
    ///
    /// This is useful when dealing with tokens that are of the [sway_core::language::ty::TyExpression] type in the AST.
    /// For example, we can then use the `return_type` field which is a [TypeId] to retrieve the declaration Token.
    pub fn declaration_of_type_id(
        &self,
        engines: &Engines,
        type_id: &TypeId,
    ) -> Option<ty::TyDecl> {
        token::ident_of_type_id(engines, type_id)
            .and_then(|decl_ident| self.try_get(&decl_ident).try_unwrap())
            .map(|item| item.value().clone())
            .and_then(|token| token.typed)
            .and_then(|typed_token| match typed_token {
                TypedAstToken::TypedDeclaration(dec) => Some(dec),
                _ => None,
            })
    }

    /// Returns the [ty::TyStructDecl] associated with the TypeId if it exists
    /// within the TokenMap.
    pub fn struct_declaration_of_type_id(
        &self,
        engines: &Engines,
        type_id: &TypeId,
    ) -> Option<ty::TyStructDecl> {
        self.declaration_of_type_id(engines, type_id)
            .and_then(|decl| match decl {
                ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                    Some((*engines.de().get_struct(&decl_id)).clone())
                }
                _ => None,
            })
    }
}

impl std::ops::Deref for TokenMap {
    type Target = DashMap<TokenIdent, Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A custom iterator for [TokenMap] that yields [TokenIdent] and [Token] pairs.
pub struct TokenMapIter<'s> {
    inner_iter: dashmap::iter::Iter<'s, TokenIdent, Token>,
}

impl<'s> Iterator for TokenMapIter<'s> {
    type Item = (TokenIdent, Token);

    /// Returns the next TokenIdent in the [TokenMap].
    ///
    /// If there are no more items, returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|item| {
            let (span, token) = item.pair();
            (span.clone(), token.clone())
        })
    }
}
