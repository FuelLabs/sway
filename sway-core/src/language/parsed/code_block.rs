use crate::{
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::parsed::AstNode,
};

use sway_types::{span::Span, Spanned};

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub contents: Vec<AstNode>,
    pub(crate) whole_block_span: Span,
}

impl EqWithEngines for CodeBlock {}
impl PartialEqWithEngines for CodeBlock {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.contents.eq(&other.contents, ctx)
    }
}

impl Spanned for CodeBlock {
    fn span(&self) -> Span {
        self.whole_block_span.clone()
    }
}
