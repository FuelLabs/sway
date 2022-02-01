use crate::{
    build_config::BuildConfig,
    error::{ok, ParseResult},
    error_recovery_parse_result,
    parser::Rule,
    CompileResult, Expression,
};

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
    ) -> CompileResult<ParseResult<Self>> {
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
            None => {
                let stmt = ReturnStatement {
                    expr: Expression::Tuple {
                        fields: vec![],
                        span,
                    },
                };
                ParseResult {
                    var_decls: vec![],
                    value: stmt,
                }
            }
            Some(expr_pair) => {
                let expr_result = check!(
                    Expression::parse_from_pair(expr_pair, config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span
                    }),
                    warnings,
                    errors
                );
                let stmt = ReturnStatement {
                    expr: expr_result.value,
                };
                ParseResult {
                    var_decls: expr_result.var_decls,
                    value: stmt,
                }
            }
        };
        ok(res, warnings, errors)
    }
}
