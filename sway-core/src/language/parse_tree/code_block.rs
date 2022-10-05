use crate::language::parse_tree::AstNode;

use sway_types::{span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub contents: Vec<AstNode>,
    pub(crate) whole_block_span: Span,
}

impl Spanned for CodeBlock {
    fn span(&self) -> Span {
        self.whole_block_span.clone()
    }
}
