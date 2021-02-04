use crate::ast::Literal;
use crate::error::CompileError;
use crate::parser::{HllParser, Rule};
use pest::iterators::Pair;

#[derive(Debug)]
pub(crate) enum Expression<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: &'sc str,
        arguments: Vec<Expression<'sc>>,
    },
    VariableExpression {
        name: &'sc str,
    },
    Unit,
}

impl<'sc> Expression<'sc> {
    pub(crate) fn parse_from_pair(expr: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut expr_iter = expr.into_inner();
        let expr = expr_iter.next().unwrap();
        if expr_iter.next().is_some() {
            return Err(CompileError::Unimplemented(Rule::op, expr.into_span()));
        }
        Expression::parse_from_pair_inner(expr)
    }

    pub(crate) fn parse_from_pair_inner(expr: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let parsed = match expr.as_rule() {
            Rule::literal_value => Expression::Literal(Literal::parse_from_pair(expr)?),
            Rule::func_app => {
                let mut func_app_parts = expr.into_inner();
                let name = func_app_parts.next().unwrap().as_str();
                let arguments = func_app_parts.next();
                let arguments = arguments.map(|x| {
                    x.into_inner()
                        .map(|x| Expression::parse_from_pair_inner(x))
                        .collect::<Result<Vec<_>, _>>()
                });
                let arguments = arguments.unwrap_or_else(|| Ok(Vec::new()))?;

                Expression::FunctionApplication { name, arguments }
            }
            Rule::var_exp => {
                let mut var_exp_parts = expr.into_inner();
                Expression::VariableExpression {
                    name: var_exp_parts.next().unwrap().as_str(),
                }
            }
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_span()
                );
                return Err(CompileError::Unimplemented(a, expr.as_span()));
            }
        };
        Ok(parsed)
    }
}
