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
pub struct Length {
    val: usize,
    span: Span,
}

impl Length {
    /// Creates a new [Length] without span annotation.
    pub fn new(val: usize) -> Self {
        Length {
            val,
            span: Span::dummy(),
        }
    }

    /// Creates a new [Length] from a numeric literal.
    /// The `span` will be set to the span of the numeric literal.
    pub fn from_numeric_literal(val: usize, numeric_literal_span: Span) -> Self {
        Length {
            val,
            span: numeric_literal_span,
        }
    }

    pub fn val(&self) -> usize {
        self.val
    }

    pub fn is_annotated(&self) -> bool {
        !self.span.is_dummy()
    }
}

impl Spanned for Length {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
