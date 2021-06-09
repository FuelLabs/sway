use crate::parser::Rule;
use crate::Ident;
use crate::{error::*, types::TypeInfo};
use pest::iterators::Pair;
use pest::Span;

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CallPath<'sc> {
    pub prefixes: Vec<Ident<'sc>>,
    pub type_suffix: Option<TypeInfo<'sc>>,
    pub suffix: Ident<'sc>,
}

impl<'sc> std::convert::From<Ident<'sc>> for CallPath<'sc> {
    fn from(other: Ident<'sc>) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
            type_suffix: None,
        }
    }
}

impl<'sc> CallPath<'sc> {
    pub(crate) fn span(&self) -> Span<'sc> {
        if self.prefixes.is_empty() {
            self.suffix.span.clone()
        } else {
            let prefixes_span = self
                .prefixes
                .iter()
                .fold(self.prefixes[0].span.clone(), |acc, sp| {
                    crate::utils::join_spans(acc, sp.span.clone())
                });
            crate::utils::join_spans(prefixes_span, self.suffix.span.clone())
        }
    }
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pairs_buf = vec![];
        let mut type_suffix = None;
        for pair in pair.clone().into_inner() {
            match pair.as_rule() {
                Rule::type_name => {
                    type_suffix = Some(eval!(
                        TypeInfo::parse_from_pair,
                        warnings,
                        errors,
                        pair,
                        TypeInfo::ErrorRecovery
                    ))
                }
                Rule::path_separator => (),
                _ => pairs_buf.push(eval!(
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    pair,
                    continue
                )),
            }
        }
        assert!(pairs_buf.len() > 0);
        let suffix = pairs_buf.pop().unwrap();
        let prefixes = pairs_buf;

        ok(
            CallPath {
                prefixes,
                type_suffix,
                suffix,
            },
            warnings,
            errors,
        )
    }
}
