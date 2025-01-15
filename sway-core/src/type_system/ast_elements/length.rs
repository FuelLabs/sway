use sway_types::{span::Span, Spanned};

use crate::language::parsed::Expression;

/// Describes a fixed length for types that need it, e.g., [crate::TypeInfo::Array].
///
/// Optionally, if the length is coming from a literal in code, the [Length]
/// also keeps the [Span] of that literal. In that case, we say that the length
/// is annotated.
///
/// E.g., in this example, the two lengths coming from the literal `3` will
/// have two different spans pointing to the two different strings "3":
///
/// ```ignore
/// fn copy(a: [u64;3], b: [u64;3])
/// ```
#[derive(Debug, Clone)]
pub enum Length {
    Literal { val: usize, span: Span },
    Expression { expr: Expression },
}

impl std::hash::Hash for Length {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Length::Literal { val, .. } => {
                val.hash(state);
            }
            Length::Expression { .. } => {
                // TODO making Expression hasheable is a lot of work (some variants are not hasheable),
                // and more than we need.
                // but using span here is dangerous
                // expr.span.hash(state);
                todo!()
            }
        }
    }
}

impl Length {
    /// Creates a new literal [Length] without span annotation.
    pub fn literal(val: usize) -> Self {
        Length::Literal {
            val,
            span: Span::dummy(),
        }
    }

    /// Creates a new literal [Length] from a numeric literal.
    /// The `span` will be set to the span of the numeric literal.
    pub fn from_numeric_literal(val: usize, numeric_literal_span: Span) -> Self {
        Length::Literal {
            val,
            span: numeric_literal_span,
        }
    }

    pub fn val(&self) -> usize {
        match self {
            Length::Literal { val, .. } => *val,
            Length::Expression { .. } => todo!(),
        }
    }

    pub fn is_annotated(&self) -> bool {
        match self {
            Length::Literal { span, .. } => span,
            Length::Expression { expr, .. } => &expr.span,
        }
        .is_dummy()
    }

    pub fn same_length_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Length::Literal { val: l_val, .. }, Length::Literal { val: r_val, .. }) => {
                l_val == r_val
            }
            (Length::Expression { expr: l_expr }, Length::Expression { expr: r_expr }) => {
                // TODO improve this
                l_expr.span == r_expr.span
            }
            x => todo!("{x:?}"),
        }
    }

    pub fn get_length_str(&self) -> String {
        match self {
            Length::Literal { val, span } => {
                if !span.is_dummy() {
                    span.as_str().to_string()
                } else {
                    format!("{val}")
                }
            }
            Length::Expression { expr } => expr.span.as_str().to_string(),
        }
    }
}

impl Spanned for Length {
    fn span(&self) -> Span {
        match self {
            Length::Literal { span, .. } => span.clone(),
            Length::Expression { expr } => expr.span.clone(),
        }
    }
}
