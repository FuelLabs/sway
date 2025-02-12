use crate::engine_threading::DebugWithEngines;
use sway_types::{span::Span, Ident, Spanned};

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
    AmbiguousVariableExpression { ident: Ident },
}

impl PartialOrd for Length {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Literal { val: l, .. }, Self::Literal { val: r, .. }) => l.partial_cmp(r),
            (
                Self::AmbiguousVariableExpression { ident: l },
                Self::AmbiguousVariableExpression { ident: r },
            ) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl Eq for Length {}

impl PartialEq for Length {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal { val: l, .. }, Self::Literal { val: r, .. }) => l == r,
            (
                Self::AmbiguousVariableExpression { ident: l },
                Self::AmbiguousVariableExpression { ident: r },
            ) => l == r,
            _ => false,
        }
    }
}

impl std::hash::Hash for Length {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::Literal { val, .. } => val.hash(state),
            Self::AmbiguousVariableExpression { ident } => ident.hash(state),
        }
    }
}

impl Length {
    pub fn discriminant_value(&self) -> usize {
        match &self {
            Self::Literal { .. } => 0,
            Self::AmbiguousVariableExpression { .. } => 1,
        }
    }

    /// Creates a new literal [Length] without span annotation.
    pub fn literal(val: usize, span: Option<Span>) -> Self {
        Self::Literal {
            val,
            span: span.unwrap_or(Span::dummy()),
        }
    }

    pub fn as_literal_val(&self) -> Option<usize> {
        match self {
            Self::Literal { val, .. } => Some(*val),
            _ => None,
        }
    }

    pub fn is_annotated(&self) -> bool {
        !self.span().is_dummy()
    }
}

impl Spanned for Length {
    fn span(&self) -> Span {
        match self {
            Self::Literal { span, .. } => span.clone(),
            Self::AmbiguousVariableExpression { ident, .. } => ident.span(),
        }
    }
}

impl DebugWithEngines for Length {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, _engines: &crate::Engines) -> std::fmt::Result {
        match self {
            Self::Literal { val, .. } => write!(f, "{val}"),
            Self::AmbiguousVariableExpression { ident } => {
                write!(f, "{}", ident.as_str())
            }
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
