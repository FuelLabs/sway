use crate::{build_config::BuildConfig, error::*, parse_tree::ident, parser::Rule, Ident};

use sway_types::span::Span;

use pest::iterators::Pair;

/// in the expression `a::b::c()`, `a` and `b` are the prefixes and `c` is the suffix.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CallPath {
    pub prefixes: Vec<Ident>,
    pub suffix: Ident,
    // If `is_absolute` is true, then this call path is an absolute path from
    // the project root namespace. If not, then it is relative to the current namespace.
    pub(crate) is_absolute: bool,
}

impl std::convert::From<Ident> for CallPath {
    fn from(other: Ident) -> Self {
        CallPath {
            prefixes: vec![],
            suffix: other,
            is_absolute: false,
        }
    }
}

use std::fmt;
impl fmt::Display for CallPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = self.prefixes.iter().map(|x| x.as_str()).collect::<Vec<_>>();
        let suffix = self.suffix.as_str();
        buf.push(suffix);

        write!(f, "{}", buf.join("::"))
    }
}
impl CallPath {
    /// shifts the last prefix into the suffix and removes the old suffix
    /// noop if prefixes are empty
    pub fn rshift(&self) -> CallPath {
        if self.prefixes.is_empty() {
            self.clone()
        } else {
            CallPath {
                prefixes: self.prefixes[0..self.prefixes.len() - 1].to_vec(),
                suffix: self.prefixes.last().unwrap().clone(),
                is_absolute: self.is_absolute,
            }
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
                    Span::join(acc, sp.span().clone())
                });
            Span::join(prefixes_span, self.suffix.span().clone())
        }
    }
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<CallPath> {
        assert!(pair.as_rule() == Rule::call_path || pair.as_rule() == Rule::call_path_);
        let mut warnings = vec![];
        let mut errors = vec![];
        let span = Span::from_pest(pair.as_span(), config.map(|c| c.path()));
        if !(pair.as_rule() == Rule::call_path || pair.as_rule() == Rule::call_path_) {
            errors.push(CompileError::ParseError {
                span,
                err: "expected call path here".to_string(),
            });
            return err(warnings, errors);
        }
        let mut pairs_buf = vec![];
        let stmt = pair.into_inner().next().unwrap();
        let is_absolute = stmt.as_rule() == Rule::absolute_call_path
            || stmt.as_rule() == Rule::absolute_call_path_;
        let stmt = stmt.into_inner();
        let it = if is_absolute {
            stmt.skip(1)
        } else {
            stmt.skip(0)
        };
        for pair in it {
            if pair.as_rule() != Rule::path_separator {
                pairs_buf.push(check!(
                    ident::parse_from_pair(pair, config),
                    continue,
                    warnings,
                    errors
                ));
            }
        }
        assert!(!pairs_buf.is_empty());
        let suffix = pairs_buf.pop().unwrap();
        let prefixes = pairs_buf;

        ok(
            CallPath {
                prefixes,
                suffix,
                is_absolute,
            },
            warnings,
            errors,
        )
    }

    pub(crate) fn friendly_name(&self) -> String {
        let mut buf = String::new();
        for prefix in self.prefixes.iter() {
            buf.push_str(prefix.as_str());
            buf.push_str("::");
        }
        buf.push_str(self.suffix.as_str());
        buf
    }
}
