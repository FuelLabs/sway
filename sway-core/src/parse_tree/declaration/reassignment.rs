use crate::parse_tree::Expression;

use sway_types::{span::Span, Ident, Spanned};

/// Represents the left hand side of a reassignment, which could either be a regular variable
/// expression, denoted by [ReassignmentTarget::VariableExpression], or, a storage field, denoted
/// by [ReassignmentTarget::StorageField].
#[derive(Debug, Clone)]
pub enum ReassignmentTarget {
    VariableExpression(Box<Expression>),
    StorageField(Vec<Ident>),
}

#[derive(Debug, Clone)]
pub struct Reassignment {
    // the thing being reassigned
    pub lhs: ReassignmentTarget,
    // the expression that is being assigned to the lhs
    pub rhs: Expression,
    pub(crate) span: Span,
}

impl Reassignment {
    pub fn lhs_span(&self) -> Span {
        match &self.lhs {
            ReassignmentTarget::VariableExpression(var) => var.span.clone(),
            ReassignmentTarget::StorageField(ref idents) => idents
                .iter()
                .fold(idents[0].span(), |acc, ident| Span::join(acc, ident.span())),
        }
    }
}
