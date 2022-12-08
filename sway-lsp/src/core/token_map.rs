use crate::core::token::{self, Token, TypedAstToken};
use dashmap::DashMap;
use sway_core::{declaration_engine, language::ty, type_system::TypeId, TypeEngine};
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

    /// Given a cursor [Position], return the [Ident] of a token in the
    /// Iterator if one exists at that position.
    pub fn ident_at_position<I>(&self, cursor_position: Position, tokens: I) -> Option<Ident>
    where
        I: Iterator<Item = (Ident, Token)>,
    {
        for (ident, _) in tokens {
            let range = token::get_range_from_span(&ident.span());
            if cursor_position >= range.start && cursor_position <= range.end {
                return Some(ident);
            }
        }
        None
    }

    /// Check if the code editor's cursor is currently over one of our collected tokens.
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(Ident, Token)> {
        let tokens = self.tokens_for_file(uri);
        match self.ident_at_position(position, tokens) {
            Some(ident) => self.get(&token::to_ident_key(&ident)).map(|item| {
                let ((ident, _), token) = item.pair();
                (ident.clone(), token.clone())
            }),
            None => None,
        }
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
        type_engine: &TypeEngine,
        type_id: &TypeId,
    ) -> Option<ty::TyStructDeclaration> {
        self.declaration_of_type_id(type_engine, type_id)
            .and_then(|decl| match decl {
                ty::TyDeclaration::StructDeclaration(ref decl_id) => {
                    declaration_engine::de_get_struct(decl_id.clone(), &decl_id.span()).ok()
                }
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
