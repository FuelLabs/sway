use crate::build_config::BuildConfig;
use crate::error::ok;
use crate::parser::Rule;
use crate::span;
use crate::{CompileResult, Expression};
use pest::iterators::Pair;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ReturnStatement<'sc> {
    pub expr: Expression<'sc>,
}

impl<'sc> ReturnStatement<'sc> {
    pub(crate) fn parse_from_pair(
        pair: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let span = span::Span {
            span: pair.as_span(),
            path: config.map(|c| c.path()),
        };
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut inner = pair.into_inner();
        let _ret_keyword = inner.next();
        let expr = inner.next();
        let res = match expr {
            None => ReturnStatement {
                expr: Expression::Unit { span },
            },
            Some(expr_pair) => {
                let expr = check!(
                    Expression::parse_from_pair(expr_pair, config, docstrings),
                    Expression::Unit { span },
                    warnings,
                    errors
                );
                ReturnStatement { expr }
            }
        };
        ok(res, warnings, errors)
    }
}
