use super::Expression;
use crate::{
    language::{AsmOp, AsmRegister},
    TypeInfo,
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct AsmExpression {
    pub(crate) registers:        Vec<AsmRegisterDeclaration>,
    pub(crate) body:             Vec<AsmOp>,
    pub(crate) returns:          Option<(AsmRegister, Span)>,
    pub(crate) return_type:      TypeInfo,
    pub(crate) whole_block_span: Span,
}

#[derive(Debug, Clone)]
pub(crate) struct AsmRegisterDeclaration {
    pub(crate) name:        Ident,
    pub(crate) initializer: Option<Expression>,
}
