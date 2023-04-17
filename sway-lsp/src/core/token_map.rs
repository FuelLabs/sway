use crate::core::token::{self, Token, TypedAstToken};
use dashmap::DashMap;
use sway_core::{language::ty, type_system::TypeId, Engines};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{Position, Url};

// Re-export the TokenMapExt trait.
pub use crate::core::token_map_ext::TokenMapExt;

/// The TokenMap is the main data structure of the language server.
/// It stores all of the tokens that have been parsed and typechecked by the sway compiler.
///
/// The TokenMap is a wrapper around a [DashMap], which is a concurrent HashMap.
#[derive(Debug)]
pub struct TokenMap(DashMap<(Ident, Span), Token>);

impl TokenMap {
    /// Create a new token map.
    pub fn new() -> TokenMap {
        TokenMap(DashMap::new())
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
    ) -> impl 's + Iterator<Item = (Ident, Token)> {
        self.iter().flat_map(|(ident, token)| {
            ident.span().path().and_then(|path| {
                if path.to_str() == Some(uri.path()) {
                    Some((ident.clone(), token.clone()))
                } else {
                    None
                }
            })
        })
    }

    /// Given a cursor [Position], return the [Ident] of a token in the
    /// Iterator if one exists at that position.
    pub fn idents_at_position<I>(&self, cursor_position: Position, tokens: I) -> Vec<Ident>
    where
        I: Iterator<Item = (Ident, Token)>,
    {
        tokens
            .filter_map(|(ident, _)| {
                let range = token::get_range_from_span(&ident.span());
                if cursor_position >= range.start && cursor_position <= range.end {
                    return Some(ident);
                }
                None
            })
            .collect()
    }

    /// Returns the first parent declaration found at the given cursor position.
    ///
    /// For example, if the cursor is inside a function body, this function returns the function declaration.
    pub fn parent_decl_at_position(&self, uri: &Url, position: Position) -> Option<(Ident, Token)> {
        self.tokens_at_position(uri, position, None)
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
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(Ident, Token)> {
        let tokens = self.tokens_for_file(uri);
        self.idents_at_position(position, tokens)
            .first()
            .and_then(|ident| {
                self.try_get(&token::to_ident_key(ident))
                    .try_unwrap()
                    .map(|item| {
                        let ((ident, _), token) = item.pair();
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
        uri: &Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<(Ident, Token)> {
        self.tokens_for_file(uri)
            .filter_map(|(ident, token)| {
                let span = match token.typed {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl))
                        if functions_only == Some(true) =>
                    {
                        decl.span()
                    }
                    Some(TypedAstToken::TypedDeclaration(decl)) => decl.span(),
                    _ => ident.span(),
                };
                let range = token::get_range_from_span(&span);
                if position >= range.start && position <= range.end {
                    return self
                        .try_get(&token::to_ident_key(&ident))
                        .try_unwrap()
                        .map(|item| {
                            let ((ident, _), token) = item.pair();
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
        engines: Engines<'_>,
        type_id: &TypeId,
    ) -> Option<ty::TyDecl> {
        token::ident_of_type_id(engines, type_id)
            .and_then(|decl_ident| self.try_get(&token::to_ident_key(&decl_ident)).try_unwrap())
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
        engines: Engines<'_>,
        type_id: &TypeId,
    ) -> Option<ty::TyStructDecl> {
        self.declaration_of_type_id(engines, type_id)
            .and_then(|decl| match decl {
                ty::TyDecl::StructDecl { decl_id, .. } => Some(engines.de().get_struct(&decl_id)),
                _ => None,
            })
    }
}

impl std::ops::Deref for TokenMap {
    type Target = DashMap<(Ident, Span), Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A custom iterator for [TokenMap] that yields [Ident] and [Token] pairs.
///
/// This iterator skips the [Span] information when iterating over the items in the [TokenMap].
pub struct TokenMapIter<'s> {
    inner_iter: dashmap::iter::Iter<'s, (Ident, Span), Token>,
}

impl<'s> Iterator for TokenMapIter<'s> {
    type Item = (Ident, Token);

    /// Returns the next (Ident, Token) pair in the [TokenMap], skipping the [Span].
    ///
    /// If there are no more items, returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|item| {
            let ((ident, _), token) = item.pair();
            (ident.clone(), token.clone())
        })
    }
}
