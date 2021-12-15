use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span::Span;
use pest::iterators::Pair;
use std::cmp::{Ord, Ordering};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct Ident<'sc> {
    pub primary_name: &'sc str,
    // sub-names are the stuff after periods
    // like x.test.thing.method()
    // `test`, `thing`, and `method` are sub-names
    // the primary name is `x`
    pub span: Span<'sc>,
}

// custom implementation of Hash so that namespacing isn't reliant on the span itself, which will
// often be different.
impl Hash for Ident<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.primary_name.hash(state);
    }
}
impl PartialEq for Ident<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.primary_name == other.primary_name
    }
}
impl Ord for Ident<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.primary_name.cmp(&other.primary_name)
    }
}

impl PartialOrd for Ident<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Ident<'_> {}

impl<'sc> Ident<'sc> {
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
        let name = pair.as_str().trim();
        ok(
            Ident {
                primary_name: name,
                span,
            },
            Vec::new(),
            Vec::new(),
        )
    }
}
