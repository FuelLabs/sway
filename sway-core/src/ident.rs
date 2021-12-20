use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::Span;
use pest::iterators::Pair;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};
use std::fmt;

/// An [Ident] is an _identifier_ with a corresponding `span` from which it was derived.
#[derive(Debug, Clone)]
pub struct Ident<'sc> {
    name_override_opt: Option<&'static str>,
    // sub-names are the stuff after periods
    // like x.test.thing.method()
    // `test`, `thing`, and `method` are sub-names
    // the primary name is `x`
    span: Span<'sc>,
}

// custom implementation of Hash so that namespacing isn't reliant on the span itself, which will
// often be different.
impl Hash for Ident<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}
impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Ord for Ident<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Ident<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Ident<'_> {}

impl<'sc> Ident<'sc> {
    pub fn as_str(&self) -> &'sc str {
        match self.name_override_opt {
            Some(name_override) => name_override,
            None => self.span.as_str(),
        }
    }

    pub fn span(&self) -> &Span<'sc> {
        &self.span
    }

    pub fn new(span: Span<'sc>) -> Ident<'sc> {
        let span = span.trim();
        Ident {
            name_override_opt: None,
            span,
        }
    }

    pub fn new_with_override(name_override: &'static str, span: Span<'sc>) -> Ident<'sc> {
        Ident {
            name_override_opt: Some(name_override),
            span,
        }
    }

    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Ident<'sc>> {
        let path = config.map(|config| config.path());
        let span = {
            let pair = pair.clone();
            if pair.as_rule() != Rule::ident {
                Span {
                    span: pair.into_inner().next().unwrap().as_span(),
                    path,
                }
            } else {
                Span {
                    span: pair.as_span(),
                    path,
                }
            }
        };
        ok(Ident::new(span), Vec::new(), Vec::new())
    }
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.as_str())
    }
}
