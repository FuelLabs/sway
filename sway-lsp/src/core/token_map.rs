use crate::core::token::{self, LspSpan, Token, TypedAstToken};
use dashmap::DashMap;
use lsp_types::{Position, Url};
use sway_core::{language::ty, type_system::TypeId, Engines};
use sway_types::{SourceEngine, Spanned};

// Re-export the TokenMapExt trait.
pub use crate::core::token_map_ext::TokenMapExt;

/// The TokenMap is the main data structure of the language server.
/// It stores all of the tokens that have been parsed and typechecked by the sway compiler.
///
/// The TokenMap is a wrapper around a [DashMap], which is a concurrent HashMap.
#[derive(Debug, Default)]
pub struct TokenMap(DashMap<LspSpan, Token>);

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
    ) -> impl 's + Iterator<Item = (LspSpan, Token)> {
        self.iter().flat_map(|(span, token)| {
            span.path.as_ref().and_then(|path| {
                if path.to_str() == Some(uri.path()) {
                    Some((span.clone(), token.clone()))
                } else {
                    None
                }
            })
        })
    }

    /// Given a cursor [Position], return the [LspSpan] of a token in the
    /// Iterator if one exists at that position.
    pub fn spans_at_position<I>(&self, cursor_position: Position, tokens: I) -> Vec<LspSpan>
    where
        I: Iterator<Item = (LspSpan, Token)>,
    {
        tokens
            .filter_map(|(span, _)| {
                if cursor_position >= span.range.start && cursor_position <= span.range.end {
                    return Some(span);
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
    ) -> Option<(LspSpan, Token)> {
        self.tokens_at_position(source_engine, uri, position, None)
            .iter()
            .find_map(|(lsp_span, token)| {
                if let Some(TypedAstToken::TypedDeclaration(_)) = token.typed {
                    Some((lsp_span.clone(), token.clone()))
                } else {
                    None
                }
            })
    }

    /// Returns the first collected tokens that is at the cursor position.
    pub fn token_at_position(&self, uri: &Url, position: Position) -> Option<(LspSpan, Token)> {
        let tokens = self.tokens_for_file(uri);
        self.spans_at_position(position, tokens)
            .first()
            .and_then(|lsp_span| {
                self.try_get(&lsp_span).try_unwrap().map(|item| {
                    let (lsp_span, token) = item.pair();
                    (lsp_span.clone(), token.clone())
                })
            })
    }

    /// Returns all collected tokens that are at the given [Position] in the file.
    /// If `functions_only` is true, it only returns tokens of type [TypedAstToken::TypedFunctionDeclaration].
    ///
    /// This is different from `spans_at_position` because this searches the spans of token bodies, not
    /// just the spans of the token idents. For example, if we want to find out what function declaration
    /// the cursor is inside of, we need to search the body of the function declaration, not just the ident
    /// of the function declaration (the function name).
    pub fn tokens_at_position(
        &self,
        source_engine: &SourceEngine,
        uri: &Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<(LspSpan, Token)> {
        self.tokens_for_file(uri)
            .filter_map(|(lsp_span, token)| {
                let span = match token.typed {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl))
                        if functions_only == Some(true) =>
                    {
                        LspSpan::new(&decl.span, source_engine)
                    }
                    Some(TypedAstToken::TypedDeclaration(decl)) => {
                        LspSpan::new(&decl.span(), source_engine)
                    }
                    _ => lsp_span.clone(),
                };
                if position >= span.range.start && position <= span.range.end {
                    return self.try_get(&lsp_span).try_unwrap().map(|item| {
                        let (lsp_span, token) = item.pair();
                        (lsp_span.clone(), token.clone())
                    });
                }
                None
            })
            .filter_map(|(lsp_span, token)| {
                if functions_only == Some(true) {
                    if let Some(TypedAstToken::TypedFunctionDeclaration(_)) = token.typed {
                        return Some((lsp_span, token));
                    }
                    return None;
                }
                Some((lsp_span, token))
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
        token::lsp_span_of_type_id(engines, type_id)
            .and_then(|decl_lsp_span| self.try_get(&decl_lsp_span).try_unwrap())
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
                    Some(engines.de().get_struct(&decl_id))
                }
                _ => None,
            })
    }
}

impl std::ops::Deref for TokenMap {
    type Target = DashMap<LspSpan, Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A custom iterator for [TokenMap] that yields [LspSpan] and [Token] pairs.
pub struct TokenMapIter<'s> {
    inner_iter: dashmap::iter::Iter<'s, LspSpan, Token>,
}

impl<'s> Iterator for TokenMapIter<'s> {
    type Item = (LspSpan, Token);

    /// Returns the next LspSpan in the [TokenMap].
    ///
    /// If there are no more items, returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|item| {
            let (span, token) = item.pair();
            (span.clone(), token.clone())
        })
    }
}
