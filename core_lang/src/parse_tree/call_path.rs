use crate::error::*;
use crate::parser::Rule;
use crate::Ident;
use pest::iterators::Pair;
use pest::Span;

/// in the expression `a.b.c()`, `a` and `b` are the prefixes and `c` is the suffix.
#[derive(Debug, Clone)]
pub(crate) struct CallPath<'sc> {
    pub(crate) prefixes: Vec<Ident<'sc>>,
    pub(crate) suffix: Ident<'sc>,
}

impl<'sc> std::convert::From<Ident<'sc>> for CallPath<'sc> {
    fn from(other: Ident<'sc>) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
        }
    }
}

impl<'sc> CallPath<'sc> {
    #[allow(dead_code)]
    pub(crate) fn span(&self) -> Span<'sc> {
        let prefixes_span = self
            .prefixes
            .iter()
            .fold(self.prefixes[0].span.clone(), |acc, sp| {
                crate::utils::join_spans(acc, sp.span.clone())
            });
        crate::utils::join_spans(prefixes_span, self.suffix.span.clone())
    }
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pairs_buf = vec![];
        for pair in pair.clone().into_inner() {
            if pair.as_rule() != Rule::path_separator {
                pairs_buf.push(eval!(
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    pair,
                    continue
                ));
            }
        }
        if pairs_buf.len() == 0 {
            dbg!(&pair);
        }
        assert!(pairs_buf.len() > 0);
        let suffix = pairs_buf.pop().unwrap();
        let prefixes = pairs_buf;

        // TODO eventually we want to be able to call methods with colon-delineated syntax
        ok(CallPath { prefixes, suffix }, warnings, errors)
    }
}
