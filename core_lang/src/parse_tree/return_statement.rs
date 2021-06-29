use crate::error::ok;
use crate::parser::Rule;
use crate::{CompileResult, Expression};
use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct ReturnStatement<'sc> {
    pub expr: Expression<'sc>,
}

impl<'sc> ReturnStatement<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let span = pair.as_span();
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
                let expr = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    expr_pair,
                    Expression::Unit { span }
                );
                ReturnStatement { expr }
            }
        };
        ok(res, warnings, errors)
    }
}
