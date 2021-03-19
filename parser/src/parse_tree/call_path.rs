use crate::CompileResult;

use super::Ident;
use crate::parser::Rule;
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
        todo!("{:#?}", pair)
    }
}
