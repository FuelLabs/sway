

use crate::{
    error::{err, ok}, CallPath, CompileError, CompileResult, Literal,
};

use sway_types::{ident::Ident, span::Span};

/// A [Scrutinee] is on the left-hand-side of a pattern, and dictates whether or
/// not a pattern will succeed at pattern matching and what, if any, elements will
/// need to be implemented in a desugared if expression.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum Scrutinee {
    Unit {
        span: Span,
    },
    Literal {
        value: Literal,
        span: Span,
    },
    Variable {
        name: Ident,
        span: Span,
    },
    StructScrutinee {
        struct_name: Ident,
        fields: Vec<StructScrutineeField>,
        span: Span,
    },
    EnumScrutinee {
        call_path: CallPath,
        variable_to_assign: Ident,
        span: Span,
    },
    Tuple {
        elems: Vec<Scrutinee>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct StructScrutineeField {
    pub field: Ident,
    pub scrutinee: Option<Scrutinee>,
    pub span: Span,
}

impl Scrutinee {
    pub fn span(&self) -> Span {
        match self {
            Scrutinee::Literal { span, .. } => span.clone(),
            Scrutinee::Unit { span } => span.clone(),
            Scrutinee::Variable { span, .. } => span.clone(),
            Scrutinee::StructScrutinee { span, .. } => span.clone(),
            Scrutinee::EnumScrutinee { span, .. } => span.clone(),
            Scrutinee::Tuple { span, .. } => span.clone(),
        }
    }

    /// If this is an enum scrutinee, returns the name of the inner value that should be
    /// assigned to upon successful destructuring.
    /// Should only be used when destructuring enums via `if let`
    pub fn enum_variable_to_assign(&self) -> CompileResult<&Ident> {
        match self {
            Scrutinee::EnumScrutinee {
                variable_to_assign, ..
            } => ok(variable_to_assign, vec![], vec![]),
            _ => err(
                vec![],
                vec![CompileError::IfLetNonEnum { span: self.span() }],
            ),
        }
    }
}
