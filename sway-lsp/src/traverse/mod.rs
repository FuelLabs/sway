use crate::core::{token::LspSpan, token_map::TokenMap};
use sway_core::{namespace::Module, Engines};

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

pub struct ParseContext<'a> {
    tokens: &'a TokenMap,
    engines: &'a Engines,
    namespace: &'a Module,
}

impl<'a> ParseContext<'a> {
    pub fn new(tokens: &'a TokenMap, engines: &'a Engines, namespace: &'a Module) -> Self {
        Self {
            tokens,
            engines,
            namespace,
        }
    }

    pub fn lsp_span(&self, span: &sway_types::Span) -> LspSpan {
        LspSpan::new(span, &self.engines.se())
    }
}

/// The `Parse` trait is used to parse tokens from an AST during traversal.
pub trait Parse {
    fn parse(&self, ctx: &ParseContext);
}
