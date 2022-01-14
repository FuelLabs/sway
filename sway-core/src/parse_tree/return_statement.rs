use crate::{build_config::BuildConfig, error::ok, parser::Rule, CompileResult, Expression};

use sway_types::span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Expression,
}

impl ReturnStatement {
    pub(crate) fn parse_from_pair(
        pair: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
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
                expr: Expression::Tuple {
                    fields: vec![],
                    span,
                },
            },
            Some(expr_pair) => {
                let expr = check!(
                    Expression::parse_from_pair(expr_pair, config),
                    Expression::Tuple {
                        fields: vec![],
                        span
                    },
                    warnings,
                    errors
                );
                ReturnStatement { expr }
            }
        };
        ok(res, warnings, errors)
    }
}
