use crate::parse_tree::Literal;
use std::hash::{Hash, Hasher};
#[macro_use]
use crate::error::{ParseError, ParseResult};
use crate::parser::{HllParser, Rule};
use crate::CodeBlock;
use either::Either;
use pest::iterators::Pair;
use pest::Span;

#[derive(Debug, Clone)]
pub(crate) enum Expression<'sc> {
    Literal {
        value: Literal<'sc>,
        span: Span<'sc>,
    },
    FunctionApplication {
        name: VarName<'sc>,
        arguments: Vec<Expression<'sc>>,
        span: Span<'sc>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
        name: VarName<'sc>,
        span: Span<'sc>,
    },
    Unit {
        span: Span<'sc>,
    },
    Array {
        contents: Vec<Expression<'sc>>,
        span: Span<'sc>,
    },
    MatchExpression {
        primary_expression: Box<Expression<'sc>>,
        branches: Vec<MatchBranch<'sc>>,
        span: Span<'sc>,
    },
    StructExpression {
        struct_name: &'sc str,
        fields: Vec<StructExpressionField<'sc>>,
        span: Span<'sc>,
    },
    CodeBlock {
        contents: CodeBlock<'sc>,
        span: Span<'sc>,
    },
    ParenthesizedExpression {
        inner: Box<Expression<'sc>>,
        span: Span<'sc>,
    },
    IfExp {
        condition: Box<Expression<'sc>>,
        then: Box<Expression<'sc>>,
        r#else: Option<Box<Expression<'sc>>>,
        span: Span<'sc>,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct StructExpressionField<'sc> {
    name: &'sc str,
    value: Expression<'sc>,
}

impl<'sc> Expression<'sc> {
    pub(crate) fn span(&self) -> Span<'sc> {
        use Expression::*;
        (match self {
            Literal { span, .. } => span,
            FunctionApplication { span, .. } => span,
            VariableExpression { span, .. } => span,
            Unit { span } => span,
            Array { span, .. } => span,
            MatchExpression { span, .. } => span,
            StructExpression { span, .. } => span,
            CodeBlock { span, .. } => span,
            ParenthesizedExpression { span, .. } => span,
            IfExp { span, .. } => span,
        })
        .clone()
    }
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
            let (expr, mut l_warnings) =
                arrange_by_order_of_operations(expr_or_op_buf, expr_for_debug.as_span())?;
            warnings.append(&mut l_warnings);
            Ok((expr, warnings))
        }
    }

    pub(crate) fn parse_from_pair_inner(expr: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let span = expr.as_span();
        let parsed = match expr.as_rule() {
            Rule::literal_value => Expression::Literal {
                value: Literal::parse_from_pair(expr.clone())?,
                span: expr.as_span(),
            },
            Rule::func_app => {
                let span = expr.as_span();
                let mut func_app_parts = expr.into_inner();
                let name = VarName::parse_from_pair(func_app_parts.next().unwrap())?;
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

                Expression::FunctionApplication {
                    name,
                    arguments,
                    span,
                }
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
                Expression::VariableExpression {
                    name,
                    unary_op,
                    span,
                }
            }
            Rule::array_exp => {
                let mut array_exps = expr.into_inner();
                let mut contents = Vec::new();
                for expr in array_exps {
                    contents.push(eval!(Expression::parse_from_pair, warnings, expr));
                }
                Expression::Array { contents, span }
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
                    span,
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
                    span,
                }
            }
            Rule::parenthesized_expression => {
                let expr = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    expr.into_inner().next().unwrap()
                );
                Expression::ParenthesizedExpression {
                    inner: Box::new(expr),
                    span,
                }
            }
            Rule::code_block => {
                let expr = eval!(crate::CodeBlock::parse_from_pair, warnings, expr);
                Expression::CodeBlock {
                    contents: expr,
                    span,
                }
            }
            Rule::if_exp => {
                let span = expr.as_span();
                let mut if_exp_pairs = expr.into_inner();
                let condition_pair = if_exp_pairs.next().unwrap();
                let then_pair = if_exp_pairs.next().unwrap();
                let else_pair = if_exp_pairs.next();
                let condition =
                    Box::new(eval!(Expression::parse_from_pair, warnings, condition_pair));
                let then = Box::new(eval!(
                    Expression::parse_from_pair_inner,
                    warnings,
                    then_pair
                ));
                let r#else = match else_pair {
                    Some(else_pair) => Some(Box::new(eval!(
                        Expression::parse_from_pair_inner,
                        warnings,
                        else_pair
                    ))),
                    None => None,
                };
                Expression::IfExp {
                    condition,
                    then,
                    r#else,
                    span,
                }
            }
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_rule()
                );
                return Err(ParseError::Unimplemented(a, expr.as_span()));
            }
        };
        Ok((parsed, warnings))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MatchBranch<'sc> {
    pub(crate) condition: MatchCondition<'sc>,
    pub(crate) result: Expression<'sc>,
    pub(crate) span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) enum MatchCondition<'sc> {
    CatchAll,
    Expression(Expression<'sc>),
}

impl<'sc> MatchBranch<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let span = pair.as_span();
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
            Rule::expr => eval!(Expression::parse_from_pair, warnings, result),
            Rule::code_block => {
                let span = result.as_span();
                Expression::CodeBlock {
                    contents: eval!(CodeBlock::parse_from_pair, warnings, result),
                    span,
                }
            }
            _ => unreachable!(),
        };
        Ok((
            MatchBranch {
                condition,
                result,
                span,
            },
            warnings,
        ))
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
    pub(crate) primary_name: &'sc str,
    // sub-names are the stuff after periods
    // like x.test.thing.method()
    // `test`, `thing`, and `method` are sub-names
    // the primary name is `x`
    pub(crate) sub_names: Vec<&'sc str>,
    pub(crate) span: Span<'sc>,
}

// custom implementation of Hash so that namespacing isn't reliant on the span itself, which will
// always be different.
impl Hash for VarName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.primary_name.hash(state);
        self.sub_names.hash(state);
    }
}
impl PartialEq for VarName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.primary_name == other.primary_name && self.sub_names == other.sub_names
    }
}

impl Eq for VarName<'_> {}

impl<'sc> VarName<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<VarName<'sc>, ParseError<'sc>> {
        let span = {
            let pair = pair.clone();
            if pair.as_rule() != Rule::ident {
                pair.into_inner().next().unwrap().as_span()
            } else {
                pair.as_span()
            }
        };
        let mut names = pair.into_inner();
        let primary_name = names.next().unwrap().as_str();
        let sub_names = names.map(|x| x.as_str()).collect();
        Ok(VarName {
            primary_name,
            sub_names,
            span,
        })
    }
}

fn parse_op<'sc>(op: Pair<'sc, Rule>) -> Result<Op, ParseError<'sc>> {
    use OpVariant::*;
    let op_variant = match op.as_str() {
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
    };
    Ok(Op {
        span: op.as_span(),
        op_variant,
    })
}

#[derive(Debug)]
struct Op<'sc> {
    span: Span<'sc>,
    op_variant: OpVariant,
}

impl<'sc> Op<'sc> {
    fn to_var_name(&self) -> VarName<'sc> {
        VarName {
            primary_name: self.op_variant.as_str(),
            span: self.span.clone(),
            sub_names: vec!["std".into(), "ops".into()],
        }
    }
}
#[derive(Debug)]
enum OpVariant {
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

impl OpVariant {
    fn as_str(&self) -> &'static str {
        use OpVariant::*;
        match self {
            Add => "add",
            Subtract => "subtract",
            Divide => "divide",
            Multiply => "multiply",
            Modulo => "modulo",
            Or => "or",
            And => "and",
            Equals => "equals",
            NotEquals => "not_equals",
            Xor => "xor",
            BinaryOr => "binary_or",
            BinaryAnd => "binary_and",
        }
    }
    fn precedence(&self) -> usize {
        use OpVariant::*;
        match self {
            Add => 1,
            Subtract => 1,
            Divide => 2,
            Multiply => 2,
            Modulo => 2,
            Or => 0,
            And => 0,
            Equals => 0,
            NotEquals => 0,
            Xor => 0,
            BinaryOr => 0,
            BinaryAnd => 0,
        }
    }
}

fn arrange_by_order_of_operations<'sc>(
    expressions: Vec<Either<Op<'sc>, Expression<'sc>>>,
    debug_span: Span<'sc>,
) -> ParseResult<'sc, Expression<'sc>> {
    let warnings = Vec::new();
    let mut expression_stack = Vec::new();
    let mut op_stack = Vec::new();

    for expr_or_op in expressions {
        match expr_or_op {
            Either::Left(op) => {
                if op.op_variant.precedence()
                    < op_stack
                        .last()
                        .map(|x: &Op| x.op_variant.precedence())
                        .unwrap_or(0)
                {
                    let rhs = expression_stack.pop();
                    let lhs = expression_stack.pop();
                    let new_op = op_stack.pop().unwrap();
                    if lhs.is_none() {
                        return Err(ParseError::Internal(
                            "Prematurely empty expression stack for left hand side.",
                            debug_span,
                        ));
                    }
                    if rhs.is_none() {
                        return Err(ParseError::Internal(
                            "Prematurely empty expression stack for right hand side.",
                            debug_span,
                        ));
                    }
                    let lhs = lhs.unwrap();
                    let rhs = rhs.unwrap();
                    expression_stack.push(Expression::FunctionApplication {
                        name: new_op.to_var_name(),
                        arguments: vec![lhs, rhs],
                        span: debug_span.clone(),
                    });
                }
                op_stack.push(op)
            }
            Either::Right(expr) => expression_stack.push(expr),
        }
    }

    // TODO precedence
    while let Some(op) = op_stack.pop() {
        let rhs = expression_stack.pop();
        let lhs = expression_stack.pop();

        if lhs.is_none() {
            return Err(ParseError::Internal(
                "Prematurely empty expression stack for left hand side.",
                debug_span,
            ));
        }
        if rhs.is_none() {
            return Err(ParseError::Internal(
                "Prematurely empty expression stack for right hand side.",
                debug_span,
            ));
        }

        let lhs = lhs.unwrap();
        let rhs = rhs.unwrap();

        expression_stack.push(Expression::FunctionApplication {
            name: op.to_var_name(),
            arguments: vec![lhs, rhs],
            span: debug_span.clone(),
        });
    }

    if expression_stack.len() != 1 {
        return Err(ParseError::Internal(
            "Invalid expression stack length",
            debug_span,
        ));
    }

    Ok((expression_stack[0].clone(), warnings))
}
