use crate::language::parsed::Expression;
use sway_types::{span::Span, Spanned};

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
#[derive(Debug, Clone, Hash)]
pub struct Length(pub LengthExpression);

#[derive(Debug, Clone)]
pub enum LengthExpression {
    Literal { val: usize, span: Span },
    AmbiguousVariableExpression { inner: Expression },
}

impl std::hash::Hash for LengthExpression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            LengthExpression::Literal { val, .. } => val.hash(state),
            LengthExpression::AmbiguousVariableExpression { inner } => match &inner.kind {
                crate::language::parsed::ExpressionKind::AmbiguousVariableExpression(
                    base_ident,
                ) => base_ident.hash(state),
                _ => unreachable!(),
            },
        }
    }
}

impl Length {
    /// Creates a new literal [Length] without span annotation.
    pub fn literal(val: usize, span: Option<Span>) -> Self {
        Length(LengthExpression::Literal {
            val,
            span: span.unwrap_or_else(|| Span::dummy()),
        })
    }

    pub fn as_literal_val(&self) -> Option<usize> {
        match self.0 {
            LengthExpression::Literal { val, .. } => Some(val),
            _ => None,
        }
    }

    pub fn is_annotated(&self) -> bool {
        !self.span().is_dummy()
    }
}

impl Spanned for Length {
    fn span(&self) -> Span {
        match &self.0 {
            LengthExpression::Literal { span, .. } => span.clone(),
            LengthExpression::AmbiguousVariableExpression { inner, .. } => inner.span(),
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct NumericLength {
    pub val: usize,
    pub span: Span,
}

impl NumericLength {
    pub fn val(&self) -> usize {
        self.val
    }

    pub fn is_annotated(&self) -> bool {
        !self.span().is_dummy()
    }
}

impl Spanned for NumericLength {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
