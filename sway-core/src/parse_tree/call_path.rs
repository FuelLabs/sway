use crate::build_config::BuildConfig;
use crate::error::*;
use crate::parser::Rule;
use crate::span::Span;
use crate::Ident;
use pest::iterators::Pair;

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CallPath {
    pub prefixes: Vec<Ident>,
    pub suffix: Ident,
}

impl std::convert::From<Ident> for CallPath {
    fn from(other: Ident) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OwnedCallPath {
    pub prefixes: Vec<String>,
    pub suffix: String,
}

impl CallPath {
    pub(crate) fn to_owned_call_path(&self) -> OwnedCallPath {
        OwnedCallPath {
            prefixes: self
                .prefixes
                .iter()
                .map(|x| x.as_str().to_string())
                .collect(),
            suffix: self.suffix.as_str().to_string(),
        }
    }
}
impl CallPath {
    pub(crate) fn span(&self) -> Span {
        if self.prefixes.is_empty() {
            self.suffix.span().clone()
        } else {
            let prefixes_span = self
                .prefixes
                .iter()
                .fold(self.prefixes[0].span().clone(), |acc, sp| {
                    crate::utils::join_spans(acc, sp.span().clone())
                });
            crate::utils::join_spans(prefixes_span, self.suffix.span().clone())
        }
    }
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<CallPath> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut pairs_buf = vec![];
        for pair in pair.into_inner() {
            if pair.as_rule() != Rule::path_separator {
                pairs_buf.push(check!(
                    Ident::parse_from_pair(pair, config),
                    continue,
                    warnings,
                    errors
                ));
            }
        }
        assert!(!pairs_buf.is_empty());
        let suffix = pairs_buf.pop().unwrap();
        let prefixes = pairs_buf;

        // TODO eventually we want to be able to call methods with colon-delineated syntax
        ok(CallPath { prefixes, suffix }, warnings, errors)
    }
}
