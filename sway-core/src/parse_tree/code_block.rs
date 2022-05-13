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

fn collect_var_decls(var_decls: Vec<VariableDeclaration>, span: Span) -> Vec<AstNode> {
    var_decls
        .into_iter()
        .map(|x| AstNode {
            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(x)),
            span: span.clone(),
        })
        .collect::<Vec<_>>()
}
