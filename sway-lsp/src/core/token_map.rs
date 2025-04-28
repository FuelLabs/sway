use crate::core::token::{self, Token, TokenIdent, TypedAstToken};
use dashmap::{
    mapref::{
        multiple::RefMulti,
        one::{Ref, RefMut},
    },
    try_result::TryResult,
    DashMap,
};
use lsp_types::{Position, Url};
use std::{path::PathBuf, thread, time::Duration};
use sway_core::{engine_threading::SpannedWithEngines, language::ty, type_system::TypeId, Engines};
use sway_types::{Ident, ProgramId};

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
    /// Retries up to 14 times with increasing backoff (1ns, 10ns, 100ns, 500ns, 1µs, 10µs, 100µs, 1ms, 10ms, 50ms, 100ms, 200ms, 500ms, 1s).
    pub fn try_get_mut_with_retry(
        &'a self,
        ident: &TokenIdent,
    ) -> Option<RefMut<'a, TokenIdent, Token>> {
        const MAX_RETRIES: usize = 14;
        let backoff_times = [
            1,
            10,
            100,
            500,
            1_000,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
            50_000_000,
            100_000_000,
            200_000_000,
            500_000_000,
            1_000_000_000,
        ]; // Backoff times in nanoseconds
        for sleep in backoff_times.iter().take(MAX_RETRIES) {
            match self.try_get_mut(ident) {
                TryResult::Present(token) => return Some(token),
                TryResult::Absent => return None,
                TryResult::Locked => {
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

    /// Return an Iterator of tokens belonging to the provided [ProgramId].
    pub fn tokens_for_program<'s>(
        &'s self,
        program_id: ProgramId,
    ) -> impl Iterator<Item = RefMulti<'s, TokenIdent, Token>> + 's {
        self.iter().filter_map(move |entry| {
            entry
                .key()
                .program_id()
                .filter(|&pid| pid == program_id)
                .map(|_| entry)
        })
    }

    /// Return an Iterator of tokens belonging to the provided [Url].
    pub fn tokens_for_file<'s>(
        &'s self,
        uri: &'s Url,
    ) -> impl Iterator<Item = RefMulti<'s, TokenIdent, Token>> + 's {
        self.iter().filter_map(move |entry| {
            let ident_path = entry.key().path.clone();
            ident_path.as_ref().and_then(|path| {
                if path.to_str() == Some(uri.path()) {
                    Some(entry)
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
    ) -> impl Iterator<Item = RefMulti<'s, TokenIdent, Token>> + 's {
        self.iter().filter_map(move |entry| {
            let ident = entry.key();
            if &ident.name == name {
                Some(entry)
            } else {
                None
            }
        })
    }

    /// Given a cursor [Position], return the [TokenIdent] of a token in the
    /// Iterator if one exists at that position.
    pub fn idents_at_position<'s, I>(
        &'s self,
        cursor_position: Position,
        tokens: I,
    ) -> Vec<TokenIdent>
    where
        I: Iterator<Item = RefMulti<'s, TokenIdent, Token>>,
    {
        tokens
            .filter_map(|entry| {
                let ident = entry.key();
                if cursor_position >= ident.range.start && cursor_position <= ident.range.end {
                    Some(ident.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the first parent declaration found at the given cursor position.
    ///
    /// For example, if the cursor is inside a function body, this function returns the function declaration.
    pub fn parent_decl_at_position<'s>(
        &'s self,
        engines: &'s Engines,
        uri: &'s Url,
        position: Position,
    ) -> Option<RefMulti<'s, TokenIdent, Token>> {
        self.tokens_at_position(engines, uri, position, None)
            .into_iter()
            .find_map(|entry| {
                let (_, token) = entry.pair();
                if let Some(TypedAstToken::TypedDeclaration(_)) = &token.as_typed() {
                    Some(entry)
                } else {
                    None
                }
            })
    }

    /// Returns the first collected tokens that is at the cursor position.
    pub fn token_at_position<'s>(
        &'s self,
        uri: &'s Url,
        position: Position,
    ) -> Option<Ref<'s, TokenIdent, Token>> {
        let tokens = self.tokens_for_file(uri);
        self.idents_at_position(position, tokens)
            .first()
            .and_then(|ident| self.try_get(ident).try_unwrap())
    }

    /// Returns all collected tokens that are at the given [Position] in the file.
    /// If `functions_only` is true, it only returns tokens of type [TypedAstToken::TypedFunctionDeclaration].
    ///
    /// This is different from `idents_at_position` because this searches the spans of token bodies, not
    /// just the spans of the token idents. For example, if we want to find out what function declaration
    /// the cursor is inside of, we need to search the body of the function declaration, not just the ident
    /// of the function declaration (the function name).
    pub fn tokens_at_position<'s>(
        &'s self,
        engines: &'s Engines,
        uri: &'s Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<RefMulti<'s, TokenIdent, Token>> {
        let source_engine = engines.se();
        self.tokens_for_file(uri)
            .filter_map(move |entry| {
                let (ident, token) = entry.pair();
                let token_ident = match &token.as_typed() {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl))
                        if functions_only == Some(true) =>
                    {
                        TokenIdent::new(&Ident::new(decl.span.clone()), source_engine)
                    }
                    Some(TypedAstToken::TypedDeclaration(decl)) => {
                        TokenIdent::new(&Ident::new(decl.span(engines)), source_engine)
                    }
                    _ => ident.clone(),
                };
                if position >= token_ident.range.start && position <= token_ident.range.end {
                    if functions_only == Some(true) {
                        if let Some(TypedAstToken::TypedFunctionDeclaration(_)) = &token.as_typed()
                        {
                            return Some(entry);
                        }
                        return None;
                    }
                    Some(entry)
                } else {
                    None
                }
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
            .and_then(|token| token.as_typed().cloned())
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

    /// Remove all tokens for the given file from the token map.
    pub fn remove_tokens_for_file(&self, path_to_remove: &PathBuf) {
        self.0
            .retain(|key, _value| (key.path.as_ref() != Some(path_to_remove)));
    }
}

impl std::ops::Deref for TokenMap {
    type Target = DashMap<TokenIdent, Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
