use sway_types::{span::Span, Spanned};

/// Describes a fixed length for types that needs it such as arrays and strings
#[derive(Debug, Clone, Hash)]
pub struct Length {
    val: usize,
    span: Span,
}

impl Length {
    pub fn new(val: usize, span: Span) -> Self {
        Length { val, span }
    }

    pub fn val(&self) -> usize {
        self.val
    }
}

impl Spanned for Length {
    fn span(&self) -> Span {
        self.span.clone()
    }
}
