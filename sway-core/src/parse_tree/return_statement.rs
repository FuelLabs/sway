use crate::{
    build_config::BuildConfig,
    error::{ok, ParserLifter},
    error_recovery_exp,
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
    ) -> CompileResult<ParserLifter<Self>> {
        let span = span::Span::from_pest(pair.as_span(), config.map(|c| c.path()));
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
                ParserLifter::empty(stmt)
            }
            Some(expr_pair) => {
                let expr_result = check!(
                    Expression::parse_from_pair(expr_pair, config),
                    ParserLifter::empty(error_recovery_exp(span)),
                    warnings,
                    errors
                );
                let stmt = ReturnStatement {
                    expr: expr_result.value,
                };
                ParserLifter {
                    var_decls: expr_result.var_decls,
                    value: stmt,
                }
            }
        };
        ok(res, warnings, errors)
    }
}
