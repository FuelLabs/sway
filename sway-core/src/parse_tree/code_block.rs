use super::WhileLoop;
use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{Expression, ReturnStatement},
    AstNode, AstNodeContent, Declaration, VariableDeclaration,
};

use sway_types::span::Span;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub contents: Vec<AstNode>,
    pub(crate) whole_block_span: Span,
}

impl CodeBlock {
    pub fn span(&self) -> &Span {
        &self.whole_block_span
    }
}
