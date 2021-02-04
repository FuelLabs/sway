use crate::ast::Literal;
use crate::error::CompileError;
use crate::parser::{HllParser, Rule};
use either::Either;
use pest::iterators::Pair;

#[derive(Debug, Clone)]
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
    Array {
        contents: Vec<Expression<'sc>>,
    },
}

impl<'sc> Expression<'sc> {
    pub(crate) fn parse_from_pair(expr: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let expr_for_debug = expr.clone();
        let mut expr_iter = expr.into_inner();
        // first expr is always here
        let first_expr = expr_iter.next().unwrap();
        let first_expr = Expression::parse_from_pair_inner(first_expr)?;
        let mut expr_or_op_buf: Vec<Either<Op, Expression>> =
            vec![Either::Right(first_expr.clone())];
        // sometimes exprs are followed by ops in the same expr
        while let Some(op) = expr_iter.next() {
            let op_str = op.as_str();
            let op = parse_op(op)?;
            // an op is necessarily followed by an expression
            let next_expr = match expr_iter.next() {
                Some(o) => Expression::parse_from_pair_inner(o)?,
                None => {
                    return Err(CompileError::ExpectedExprAfterOp {
                        op: op_str,
                        span: expr_for_debug.as_span(),
                    })
                }
            };
            // pushing these into a vec in this manner so we can re-associate according to order of
            // operations later
            expr_or_op_buf.push(Either::Left(op));
            expr_or_op_buf.push(Either::Right(next_expr));
            /*
             * TODO
             * strategy: keep parsing until we have all of the op expressions
             * re-associate the expr tree with operator precedence
             */
        }
        if expr_or_op_buf.len() == 1 {
            Ok(first_expr)
        } else {
            eprintln!("Haven't yet implemented operator precedence");
            Err(CompileError::Unimplemented(
                Rule::op,
                expr_for_debug.into_span(),
            ))
        }
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
            Rule::array_exp => {
                let mut array_exps = expr.into_inner();
                Expression::Array {
                    contents: array_exps
                        .into_iter()
                        .map(|expr| Expression::parse_from_pair(expr))
                        .collect::<Result<_, _>>()?,
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

fn parse_op<'sc>(op: Pair<'sc, Rule>) -> Result<Op, CompileError<'sc>> {
    use Op::*;
    Ok(match op.as_str() {
        "+" => Add,
        "-" => Subtract,
        "/" => Divide,
        "*" => Multiply,
        "%" => Modulo,
        "||" => Or,
        "&&" => And,
        "==" => Equals,
        "!=" => NotEquals,
        "^" => Xor,
        "|" => BinaryOr,
        "&" => BinaryAnd,
        a => {
            return Err(CompileError::ExpectedOp {
                op: a,
                span: op.as_span(),
            })
        }
    })
}

#[derive(Debug)]
enum Op {
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulo,
    Or,
    And,
    Equals,
    NotEquals,
    Xor,
    BinaryOr,
    BinaryAnd,
}
