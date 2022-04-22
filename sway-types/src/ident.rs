use crate::span::Span;

use std::{
    cmp::{Ord, Ordering},
    fmt,
    hash::{Hash, Hasher},
};

/// An [Ident] is an _identifier_ with a corresponding `span` from which it was derived.
#[derive(Debug, Clone)]
pub struct Ident {
    name_override_opt: Option<&'static str>,
    span: Span,
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

impl Ident {
    pub fn as_str(&self) -> &str {
        match self.name_override_opt {
            Some(name_override) => name_override,
            None => self.span.as_str(),
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn new(span: Span) -> Ident {
        let span = span.trim();
        Ident {
            name_override_opt: None,
            span,
        }
    }

    pub fn new_with_override(name_override: &'static str, span: Span) -> Ident {
        Ident {
            name_override_opt: Some(name_override),
            span,
        }
    }

    pub fn new_no_span(name: &'static str) -> Ident {
        Ident {
            name_override_opt: Some(name),
            span: Span::dummy(),
        }
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}
