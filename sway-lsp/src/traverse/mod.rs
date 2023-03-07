use crate::core::token_map::TokenMap;
use sway_core::Engines;

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

pub struct ParseContext<'a> {
    tokens: &'a TokenMap,
    engines: Engines<'a>,
}

impl<'a> ParseContext<'a> {
    pub fn new(tokens: &'a TokenMap, engines: Engines<'a>) -> Self {
        Self { tokens, engines }
    }
}

/// The `Parse` trait is used to parse tokens from an AST during traversal.
pub trait Parse {
    fn parse(&self, ctx: &ParseContext);
}
