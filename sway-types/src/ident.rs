use crate::{span::Span, Spanned};

use std::{
    cmp::{Ord, Ordering},
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone)]
pub struct BaseIdent {
    name_override_opt: Option<&'static str>,
    span: Span,
    is_raw_ident: bool,
}

impl BaseIdent {
    pub fn as_str(&self) -> &str {
        self.name_override_opt.unwrap_or_else(|| self.span.as_str())
    }

    pub fn is_raw_ident(&self) -> bool {
        self.is_raw_ident
    }

    pub fn name_override_opt(&self) -> Option<&'static str> {
        self.name_override_opt
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

/// An [Ident] is an _identifier_ with a corresponding `span` from which it was derived.
/// It relies on a custom implementation of Hash which only looks at its textual name
/// representation, so that namespacing isn't reliant on the span itself, which will
/// often be different.
pub type Ident = BaseIdent;

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

/// An [IdentUnique] is an _identifier_ with a corresponding `span` from which it was derived.
/// Its hash and equality implementation takes the full span into account, meaning that identifiers
/// are considered unique if they originate from different files.
#[derive(Debug, Clone)]
pub struct IdentUnique(BaseIdent);

impl From<Ident> for IdentUnique {
    fn from(item: Ident) -> Self {
        IdentUnique(item)
    }
}

impl From<&Ident> for IdentUnique {
    fn from(item: &Ident) -> Self {
        IdentUnique(item.clone())
    }
}

impl From<&IdentUnique> for Ident {
    fn from(item: &IdentUnique) -> Self {
        Ident {
            name_override_opt: item.0.name_override_opt(),
            span: item.0.span(),
            is_raw_ident: item.0.is_raw_ident(),
        }
    }
}

impl Hash for IdentUnique {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.span().hash(state);
    }
}

impl PartialEq for IdentUnique {
    fn eq(&self, other: &Self) -> bool {
        self.0.span() == other.0.span()
    }
}

impl Ord for IdentUnique {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.span().cmp(&other.0.span())
    }
}

impl PartialOrd for IdentUnique {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IdentUnique {}
