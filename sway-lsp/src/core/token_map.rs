use crate::core::token::{self, Token, TypedAstToken};
use dashmap::DashMap;
use sway_core::{language::ty, type_system::TypeId, Engines, TypeEngine};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{Position, Url};

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

    /// Return an Iterator of tokens belonging to the provided [Url].
    pub fn tokens_for_file<'s>(
        &'s self,
        uri: &'s Url,
    ) -> impl 's + Iterator<Item = (Ident, Token)> {
        self.iter().flat_map(|item| {
            let ((ident, span), token) = item.pair();
            span.path().and_then(|path| {
                if path.to_str() == Some(uri.path()) {
                    Some((ident.clone(), token.clone()))
                } else {
                    None
                }
            })
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

    /// Returns the first collected tokens that is at the cursor position.
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(Ident, Token)> {
        let tokens = self.tokens_for_file(uri);
        self.idents_at_position(position, tokens)
            .first()
            .and_then(|ident| {
                self.try_get(&token::to_ident_key(&ident))
                    .try_unwrap()
                    .map(|item| {
                        let ((ident, _), token) = item.pair();
                        (ident.clone(), token.clone())
                    })
            })
    }

    /// Returns all collected tokens that are at the cursor position.
    pub fn tokens_at_position(
        &self,
        uri: &Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<(Ident, Token)> {
        let tokens = self.tokens_for_file(uri);
        tokens
            .filter_map(|(ident, token)| {
                let span = match token.typed {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl)) => decl.span(),
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

    /// Uses the [TypeId] to find the associated [ty::TyDeclaration] in the TokenMap.
    ///
    /// This is useful when dealing with tokens that are of the [sway_core::language::ty::TyExpression] type in the AST.
    /// For example, we can then use the `return_type` field which is a [TypeId] to retrieve the declaration Token.
    pub fn declaration_of_type_id(
        &self,
        type_engine: &TypeEngine,
        type_id: &TypeId,
    ) -> Option<ty::TyDeclaration> {
        token::ident_of_type_id(type_engine, type_id)
            .and_then(|decl_ident| self.try_get(&token::to_ident_key(&decl_ident)).try_unwrap())
            .map(|item| item.value().clone())
            .and_then(|token| token.typed)
            .and_then(|typed_token| match typed_token {
                TypedAstToken::TypedDeclaration(dec) => Some(dec),
                _ => None,
            })
    }

    /// Returns the [ty::TyStructDeclaration] associated with the TypeId if it
    /// exists within the TokenMap.
    pub fn struct_declaration_of_type_id(
        &self,
        engines: Engines<'_>,
        type_id: &TypeId,
    ) -> Option<ty::TyStructDeclaration> {
        let type_engine = engines.te();
        let decl_engine = engines.de();
        self.declaration_of_type_id(type_engine, type_id)
            .and_then(|decl| match decl {
                ty::TyDeclaration::StructDeclaration {
                    decl_id, decl_span, ..
                } => decl_engine.get_struct(&decl_id, &decl_span).ok(),
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
