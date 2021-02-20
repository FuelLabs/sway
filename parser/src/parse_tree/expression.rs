use crate::parse_tree::Literal;
#[macro_use]
use crate::error::{ParseError, ParseResult};
use crate::parser::{HllParser, Rule};
use crate::CodeBlock;
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
        unary_op: Option<UnaryOp>,
        name: VarName<'sc>,
    },
    Unit,
    Array {
        contents: Vec<Expression<'sc>>,
    },
    MatchExpression {
        primary_expression: Box<Expression<'sc>>,
        branches: Vec<MatchBranch<'sc>>,
    },
    StructExpression {
        struct_name: &'sc str,
        fields: Vec<StructExpressionField<'sc>>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct StructExpressionField<'sc> {
    name: &'sc str,
    value: Expression<'sc>,
}

impl<'sc> Expression<'sc> {
    pub(crate) fn parse_from_pair(expr: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let expr_for_debug = expr.clone();
        let mut expr_iter = expr.into_inner();
        // first expr is always here
        let first_expr = expr_iter.next().unwrap();
        let first_expr = eval!(Expression::parse_from_pair_inner, warnings, first_expr);
        let mut expr_or_op_buf: Vec<Either<Op, Expression>> =
            vec![Either::Right(first_expr.clone())];
        // sometimes exprs are followed by ops in the same expr
        while let Some(op) = expr_iter.next() {
            let op_str = op.as_str();
            let op = parse_op(op)?;
            // an op is necessarily followed by an expression
            let next_expr = match expr_iter.next() {
                Some(o) => eval!(Expression::parse_from_pair_inner, warnings, o),
                None => {
                    return Err(ParseError::ExpectedExprAfterOp {
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
            Ok((first_expr, warnings))
        } else {
            eprintln!("Haven't yet implemented operator precedence");
            Err(ParseError::Unimplemented(
                Rule::op,
                expr_for_debug.into_span(),
            ))
        }
    }

    pub(crate) fn parse_from_pair_inner(expr: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
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

                let mut arguments = arguments.unwrap_or_else(|| Ok(Vec::new()))?;
                let mut local_warnings = arguments.iter_mut().map(|(_, x)| x.clone());
                let mut warn_buf = Vec::new();
                for mut warning in local_warnings {
                    warn_buf.append(&mut warning);
                }
                warnings.append(&mut warn_buf);

                let arguments = arguments.into_iter().map(|(x, _)| x).collect();

                Expression::FunctionApplication { name, arguments }
            }
            Rule::var_exp => {
                let mut var_exp_parts = expr.into_inner();
                // this means that this is something like `!`, `ref`, or `deref` and the next
                // token is the actual expr value
                let mut unary_op = None;
                let mut name = None;
                while let Some(pair) = var_exp_parts.next() {
                    match pair.as_rule() {
                        Rule::unary_op => {
                            unary_op = Some(UnaryOp::parse_from_pair(pair)?);
                        }
                        Rule::var_name_ident => {
                            name = Some(VarName::parse_from_pair(pair)?);
                        }
                        a => unreachable!("what is this? {:?} {}", a, pair.as_str()),
                    }
                }
                // this is non-optional and part of the parse rule so it won't fail
                let name = name.unwrap();
                Expression::VariableExpression { name, unary_op }
            }
            Rule::array_exp => {
                let mut array_exps = expr.into_inner();
                let mut contents = Vec::new();
                for expr in array_exps {
                    contents.push(eval!(Expression::parse_from_pair, warnings, expr));
                }
                Expression::Array { contents }
            }
            Rule::match_expression => {
                let mut expr_iter = expr.into_inner();
                let primary_expression = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    expr_iter.next().unwrap()
                );
                let primary_expression = Box::new(primary_expression);
                let mut branches = Vec::new();
                for exp in expr_iter {
                    let res = eval!(MatchBranch::parse_from_pair, warnings, exp);
                    branches.push(res);
                }
                Expression::MatchExpression {
                    primary_expression,
                    branches,
                }
            }
            Rule::struct_expression => {
                let mut expr_iter = expr.into_inner();
                let struct_name = expr_iter.next().unwrap().as_str();
                let fields = expr_iter.next().unwrap().into_inner().collect::<Vec<_>>();
                let mut fields_buf = Vec::new();
                for i in (0..fields.len()).step_by(2) {
                    let name = fields[i].as_str();
                    let value = eval!(Expression::parse_from_pair, warnings, fields[i + 1].clone());
                    fields_buf.push(StructExpressionField { name, value });
                }
                // TODO add warning for capitalization on struct name
                Expression::StructExpression {
                    struct_name,
                    fields: fields_buf,
                }
            }
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_span()
                );
                return Err(ParseError::Unimplemented(a, expr.as_span()));
            }
        };
        Ok((parsed, warnings))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MatchBranch<'sc> {
    condition: MatchCondition<'sc>,
    result: Either<CodeBlock<'sc>, Expression<'sc>>,
}

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition<'sc> {
    CatchAll,
    Expression(Expression<'sc>),
}

impl<'sc> MatchBranch<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut branch = pair.clone().into_inner();
        let condition = match branch.next() {
            Some(o) => o,
            None => {
                return Err(ParseError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    pair.as_span(),
                ))
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                let expr = eval!(Expression::parse_from_pair, warnings, e);
                MatchCondition::Expression(expr)
            }
            // the "_" case
            None => MatchCondition::CatchAll,
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                return Err(ParseError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    pair.as_span(),
                ))
            }
        };
        let result = match result.as_rule() {
            Rule::expr => {
                let expr = eval!(Expression::parse_from_pair, warnings, result);
                Either::Right(expr)
            }
            Rule::code_block => Either::Left(CodeBlock::parse_from_pair(result)?),
            _ => unreachable!(),
        };
        Ok((MatchBranch { condition, result }, warnings))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    fn parse_from_pair<'sc>(pair: Pair<'sc, Rule>) -> Result<Self, ParseError<'sc>> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => Ok(Not),
            "ref" => Ok(Ref),
            "deref" => Ok(Deref),
            _ => Err(ParseError::Internal(
                "Attempted to parse unary op from invalid op string.",
                pair.as_span(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VarName<'sc> {
    primary_name: &'sc str,
    // sub-names are the stuff after periods
    // like x.test.thing.method()
    // `test`, `thing`, and `method` are sub-names
    // the primary name is `x`
    sub_names: Vec<&'sc str>,
}

impl<'sc> VarName<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<VarName<'sc>, ParseError<'sc>> {
        let mut names = pair.into_inner();
        let primary_name = names.next().unwrap().as_str();
        let sub_names = names.map(|x| x.as_str()).collect();
        Ok(VarName {
            primary_name,
            sub_names,
        })
    }
}

fn parse_op<'sc>(op: Pair<'sc, Rule>) -> Result<Op, ParseError<'sc>> {
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
            return Err(ParseError::ExpectedOp {
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
