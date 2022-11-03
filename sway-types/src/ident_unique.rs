use crate::{span::Span, Ident, Spanned};

use std::{cmp::Ord, fmt, hash::Hash};

/// An [IdentUnique] is an _identifier_ with a corresponding `span` from which it was derived.
#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Ord, Eq)]
pub struct IdentUnique {
    name_override_opt: Option<&'static str>,
    span: Span,
    is_raw_ident: bool,
}

impl From<Ident> for IdentUnique {
    fn from(item: Ident) -> Self {
        IdentUnique {
            name_override_opt: item.name_override_opt(),
            span: item.span(),
            is_raw_ident: item.is_raw_ident(),
        }
    }
}

impl From<&Ident> for IdentUnique {
    fn from(item: &Ident) -> Self {
        IdentUnique::from(item.clone())
    }
}

impl Spanned for IdentUnique {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl fmt::Display for IdentUnique {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}

impl IdentUnique {
    pub fn as_str(&self) -> &str {
        self.name_override_opt.unwrap_or_else(|| self.span.as_str())
    }

    pub fn is_raw_ident(&self) -> bool {
        self.is_raw_ident
    }

    pub fn name_override_opt(&self) -> Option<&'static str> {
        self.name_override_opt
    }
}
