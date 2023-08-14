use crate::core::token::{Token, TokenIdent};
use dashmap::DashMap;
use sway_core::{namespace::Module, Engines};

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

pub struct ParseContext<'a> {
    tokens: &'a DashMap<TokenIdent, Token>,
    engines: &'a Engines,
    namespace: &'a Module,
}

impl<'a> ParseContext<'a> {
    pub fn new(tokens: &'a DashMap<TokenIdent, Token>, engines: &'a Engines, namespace: &'a Module) -> Self {
        Self {
            tokens,
            engines,
            namespace,
        }
    }

    pub fn ident(&self, ident: &sway_types::Ident) -> TokenIdent {
        TokenIdent::new(ident, &self.engines.se())
    }
}

/// The `Parse` trait is used to parse tokens from an AST during traversal.
pub trait Parse {
    fn parse(&self, ctx: &ParseContext);
}
