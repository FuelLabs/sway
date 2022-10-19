use crate::{span::Span, Spanned};

use std::{
    cmp::{Ord, Ordering},
    fmt,
    hash::{Hash, Hasher},
};

/// An [Ident] is an _identifier_ with a corresponding `span` from which it was derived.
#[derive(Clone)]
pub struct Ident {
    name_override_opt: Option<&'static str>,
    span: Span,
    is_raw_ident: bool,
}

impl fmt::Debug for Ident {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

// custom implementation of Hash so that namespacing isn't reliant on the span itself, which will
// often be different.
impl Hash for Ident {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Ident {}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl Ident {
    pub fn as_str(&self) -> &str {
        self.name_override_opt.unwrap_or_else(|| self.span.as_str())
    }

    pub fn is_raw_ident(&self) -> bool {
        self.is_raw_ident
    }

    pub fn new(span: Span) -> Ident {
        let span = span.trim();
        Ident {
            name_override_opt: None,
            span,
            is_raw_ident: false,
        }
    }

    pub fn new_no_trim(span: Span) -> Ident {
        Ident {
            name_override_opt: None,
            span,
            is_raw_ident: false,
        }
    }

    pub fn new_with_raw(span: Span, is_raw_ident: bool) -> Ident {
        let span = span.trim();
        Ident {
            name_override_opt: None,
            span,
            is_raw_ident,
        }
    }

    pub fn new_with_override(name_override: &'static str, span: Span) -> Ident {
        Ident {
            name_override_opt: Some(name_override),
            span,
            is_raw_ident: false,
        }
    }

    pub fn new_no_span(name: &'static str) -> Ident {
        Ident {
            name_override_opt: Some(name),
            span: Span::dummy(),
            is_raw_ident: false,
        }
    }
}
