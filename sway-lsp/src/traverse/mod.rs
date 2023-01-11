use crate::core::token_map::TokenMap;

pub(crate) mod dependency;
pub(crate) mod lexed_tree;
pub(crate) mod parsed_tree;
pub(crate) mod typed_tree;

/// The `Parse` trait is used to parse tokens from an AST during traversal.
pub trait Parse {
    fn parse(&self, tokens: &TokenMap);
}
