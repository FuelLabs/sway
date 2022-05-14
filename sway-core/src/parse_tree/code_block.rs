
use crate::{
    AstNode,
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
