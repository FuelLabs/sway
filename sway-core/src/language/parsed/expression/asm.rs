use super::Expression;
use crate::{
    engine_threading::{EqWithEngines, PartialEqWithEngines, PartialEqWithEnginesContext},
    language::{AsmOp, AsmRegister},
    TypeInfo,
};
use sway_types::{ident::Ident, span::Span};

#[derive(Debug, Clone)]
pub struct AsmExpression {
    pub registers: Vec<AsmRegisterDeclaration>,
    pub(crate) body: Vec<AsmOp>,
    pub(crate) returns: Option<(AsmRegister, Span)>,
    pub(crate) return_type: TypeInfo,
    pub(crate) whole_block_span: Span,
}

impl EqWithEngines for AsmExpression {}
impl PartialEqWithEngines for AsmExpression {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.registers.eq(&other.registers, ctx)
            && self.body == other.body
            && self.returns == other.returns
            && self.return_type.eq(&other.return_type, ctx)
    }
}

#[derive(Debug, Clone)]
pub struct AsmRegisterDeclaration {
    pub(crate) name: Ident,
    pub initializer: Option<Expression>,
}

impl EqWithEngines for AsmRegisterDeclaration {}
impl PartialEqWithEngines for AsmRegisterDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.name == other.name && self.initializer.eq(&other.initializer, ctx)
    }
}
