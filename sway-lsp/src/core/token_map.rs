use crate::core::token::{self, Token, TokenIdent, TypedAstToken};
use dashmap::DashMap;
use lsp_types::{Position, Url};
use sway_core::{language::ty, type_system::TypeId, Engines};
use sway_types::{Ident, SourceEngine, Spanned};
use std::collections::HashMap;

// Re-export the TokenMapExt trait.
pub use crate::core::token_map_ext::TokenMapExt;

/// The TokenMap is the main data structure of the language server.
/// It stores all of the tokens that have been parsed and typechecked by the sway compiler.
///
/// The TokenMap is a wrapper around a [HashMap].
#[derive(Debug, Default)]
pub struct TokenMap(HashMap<TokenIdent, Token>);

impl TokenMap {
    /// Create a new token map.
    pub fn new() -> TokenMap {
        TokenMap(HashMap::new())
    }

    // /// Create a custom iterator for the TokenMap.
    // ///
    // /// The iterator returns ([Ident], [Token]) pairs.
    // pub fn iter(&self) -> TokenMapIter {
    //     TokenMapIter {
    //         inner_iter: self.0.iter(),
    //     }
    // }

    /// Return an Iterator of tokens belonging to the provided [Url].
    // pub fn tokens_for_file<'s>(
    //     &'s self,
    //     uri: &'s Url,
    // ) -> impl 's + Iterator<Item = (TokenIdent, Token)> {
    //     self.iter().flat_map(|(ident, token)| {
    //         ident.path.as_ref().and_then(|path| {
    //             if path.to_str() == Some(uri.path()) {
    //                 Some((ident.clone(), token.clone()))
    //             } else {
    //                 None
    //             }
    //         })
    //     })
    // }

    /// Return an Iterator of tokens belonging to the provided [Url].
    pub fn tokens_for_file<'s>(&'s self, uri: &'s Url) -> impl Iterator<Item = (&'s TokenIdent, &'s Token)> + 's {
        self.iter().filter(|(ident, _)| {
            ident.path.as_ref().map_or(false, |path| path.to_str() == Some(uri.path()))
        })
    }

    /// Given a cursor [Position], return the [TokenIdent] of a token in the
    /// Iterator if one exists at that position.
    pub fn idents_at_position<'s, I>(&self, cursor_position: Position, tokens: I) -> Vec<&'s TokenIdent>
    where
        I: Iterator<Item = (&'s TokenIdent, &'s Token)>,
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
    ) -> Option<(&TokenIdent, &Token)> {
        self.tokens_at_position(source_engine, uri, position, None)
            .iter()
            .find_map(|(ident, token)| {
                if let Some(TypedAstToken::TypedDeclaration(_)) = token.typed {
                    Some((*ident, *token))
                } else {
                    None
                }
            })
    }

    /// Returns the first collected tokens that is at the cursor position.
    pub fn token_at_position<'s>(&'s self, uri: &Url, position: Position) -> Option<(&'s TokenIdent, &'s Token)> {
        let tokens = self.tokens_for_file(uri);
        self.idents_at_position(position, tokens)
            .first()
            .and_then(|ident| {
                self.get(&ident).map(|token| {
                    // let (ident, token) = item.pair();
                    (*ident, token)
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
    pub fn tokens_at_position<'s>(
        &'s self,
        source_engine: &SourceEngine,
        uri: &Url,
        position: Position,
        functions_only: Option<bool>,
    ) -> Vec<(&'s TokenIdent, &'s Token)> {
        self.tokens_for_file(uri)
            .filter_map(|(ident, token)| {
                let token_ident = match token.typed {
                    Some(TypedAstToken::TypedFunctionDeclaration(decl))
                        if functions_only == Some(true) =>
                    {
                        TokenIdent::new(&Ident::new(decl.span.clone()), source_engine)
                    }
                    Some(TypedAstToken::TypedDeclaration(decl)) => {
                        TokenIdent::new(&Ident::new(decl.span().clone()), source_engine)
                    }
                    _ => ident.clone(),
                };
                if position >= token_ident.range.start && position <= token_ident.range.end {
                    return self.get(&ident).map(|token| {
                        // let (ident, token) = item.pair();
                        (ident, token)
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

   
}

impl std::ops::Deref for TokenMap {
    type Target = HashMap<TokenIdent, Token>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TokenMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A custom iterator for [TokenMap] that yields [TokenIdent] and [Token] pairs.
pub struct TokenMapIter<'s> {
    inner_iter: dashmap::iter::Iter<'s, TokenIdent, Token>,
}

impl<'s> Iterator for TokenMapIter<'s> {
    type Item = (&'s TokenIdent, &'s Token);

    /// Returns the next TokenIdent in the [TokenMap].
    ///
    /// If there are no more items, returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iter.next().map(|ref item| {
            //let (ident, token) = item.pair();
            // (ident, token)
            (item.key(), item.value())
        })
    }
}
