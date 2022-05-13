use crate::{
    build_config::BuildConfig,
    error::{err, ok, CompileError, CompileResult, ParserLifter},
    error_recovery_exp, parse_array_index,
    parse_tree::{ident, Expression},
};

use sway_types::{span::Span, Ident};

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
            ReassignmentTarget::VariableExpression(var) => match **var {
                Expression::SubfieldExpression { ref span, .. } => span.clone(),
                Expression::VariableExpression { ref name, .. } => name.span().clone(),
                _ => {
                    unreachable!("any other reassignment lhs is invalid and cannot be constructed.")
                }
            },
            ReassignmentTarget::StorageField(ref idents) => {
                idents.iter().fold(idents[0].span().clone(), |acc, ident| {
                    Span::join(acc, ident.span().clone())
                })
            }
        }
    }
}
