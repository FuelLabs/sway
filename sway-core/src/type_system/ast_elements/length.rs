use super::type_parameter::ConstGenericExpr;
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
#[derive(Debug, Clone)]
pub struct Length(pub ConstGenericExpr);

impl Length {
    pub fn expr(&self) -> &ConstGenericExpr {
        &self.0
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
