use crate::error::*;
use crate::parse_tree::{CallPath, Literal};
use crate::parser::Rule;
use crate::{CodeBlock, Ident};
use either::Either;
use pest::iterators::Pair;
use pest::Span;
use std::collections::{HashMap, VecDeque};

mod asm;
use crate::utils::join_spans;
pub(crate) use asm::*;

#[derive(Debug, Clone)]
pub enum Expression<'sc> {
    Literal {
        value: Literal<'sc>,
        span: Span<'sc>,
    },
    FunctionApplication {
        name: CallPath<'sc>,
        arguments: Vec<Expression<'sc>>,
        span: Span<'sc>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
        name: Ident<'sc>,
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
        struct_name: Ident<'sc>,
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
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span<'sc>,
        asm: AsmExpression<'sc>,
    },
    MethodApplication {
        subfield_exp: Vec<Ident<'sc>>,
        method_name: CallPath<'sc>,
        arguments: Vec<Expression<'sc>>,
        span: Span<'sc>,
    },
    /// A subfield expression is anything of the form:
    /// ```ignore
    /// <ident>.<ident>
    /// ```
    ///
    /// Where there are `n >=2` idents. This is typically an access of a structure field.
    SubfieldExpression {
        name_parts: Vec<Ident<'sc>>,
        span: Span<'sc>,
        unary_op: Option<UnaryOp>,
    },
    /// A [DelineatedPath] is anything of the form:
    /// ```ignore
    /// <ident>::<ident>
    /// ```
    /// Where there are `n >= 2` idents.
    /// These could be either enum variant constructions, or they could be
    /// references to some sort of module in the module tree.
    /// For example, a reference to a module:
    /// ```ignore
    /// std::ops::add
    /// ```
    ///
    /// And, an enum declaration:
    /// ```ignore
    /// enum MyEnum {
    ///   Variant1,
    ///   Variant2
    /// }
    ///
    /// MyEnum::Variant1
    /// ```
    DelineatedPath {
        call_path: CallPath<'sc>,
        instantiator: Option<Box<Expression<'sc>>>,
        span: Span<'sc>,
        type_arguments: Vec<crate::types::TypeInfo<'sc>>,
    },
}

#[derive(Debug, Clone)]
pub struct StructExpressionField<'sc> {
    pub(crate) name: Ident<'sc>,
    pub(crate) value: Expression<'sc>,
    pub(crate) span: Span<'sc>,
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
            AsmExpression { span, .. } => span,
            MethodApplication { span, .. } => span,
            SubfieldExpression { span, .. } => span,
            DelineatedPath { span, .. } => span,
        })
        .clone()
    }
    pub(crate) fn parse_from_pair(expr: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_for_debug = expr.clone();
        let mut expr_iter = expr.into_inner();
        // first expr is always here
        let first_expr = expr_iter.next().unwrap();
        let first_expr = eval!(
            Expression::parse_from_pair_inner,
            warnings,
            errors,
            first_expr,
            Expression::Unit {
                span: first_expr.as_span()
            }
        );
        let mut expr_or_op_buf: Vec<Either<Op, Expression>> =
            vec![Either::Right(first_expr.clone())];
        // sometimes exprs are followed by ops in the same expr
        while let Some(op) = expr_iter.next() {
            let op_str = op.as_str();
            let op_span = op.as_span();
            let op = match parse_op(op) {
                CompileResult::Ok {
                    warnings: mut l_w,
                    value,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    value
                }
                CompileResult::Err {
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    return err(warnings, errors);
                }
            };
            // an op is necessarily followed by an expression
            let next_expr = match expr_iter.next() {
                Some(o) => eval!(
                    Expression::parse_from_pair_inner,
                    warnings,
                    errors,
                    o,
                    Expression::Unit { span: o.as_span() }
                ),
                None => {
                    errors.push(CompileError::ExpectedExprAfterOp {
                        op: op_str,
                        span: expr_for_debug.as_span(),
                    });
                    Expression::Unit { span: op_span }
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
            ok(first_expr, warnings, errors)
        } else {
            let expr =
                match arrange_by_order_of_operations(expr_or_op_buf, expr_for_debug.as_span()) {
                    CompileResult::Ok {
                        value,
                        warnings: mut l_w,
                        errors: mut l_e,
                    } => {
                        warnings.append(&mut l_w);
                        errors.append(&mut l_e);
                        value
                    }
                    CompileResult::Err {
                        warnings: mut l_w,
                        errors: mut l_e,
                    } => {
                        warnings.append(&mut l_w);
                        errors.append(&mut l_e);
                        Expression::Unit {
                            span: expr_for_debug.as_span(),
                        }
                    }
                };
            ok(expr, warnings, errors)
        }
    }

    pub(crate) fn parse_from_pair_inner(expr: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = expr.as_span();
        let parsed = match expr.as_rule() {
            Rule::literal_value => match Literal::parse_from_pair(expr.clone()) {
                CompileResult::Ok {
                    value: (value, span),
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    Expression::Literal { value, span }
                }
                CompileResult::Err {
                    warnings: mut l_w,
                    errors: mut l_e,
                } => {
                    warnings.append(&mut l_w);
                    errors.append(&mut l_e);
                    Expression::Unit { span }
                }
            },
            Rule::func_app => {
                let span = expr.as_span();
                let mut func_app_parts = expr.into_inner();
                let name = eval!(
                    CallPath::parse_from_pair,
                    warnings,
                    errors,
                    func_app_parts.next().unwrap(),
                    return err(warnings, errors)
                );
                let arguments = func_app_parts.next().unwrap();
                let mut arguments_buf = Vec::new();
                for argument in arguments.into_inner() {
                    let arg = eval!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        argument,
                        Expression::Unit {
                            span: argument.as_span()
                        }
                    );
                    arguments_buf.push(arg);
                }

                Expression::FunctionApplication {
                    name,
                    arguments: arguments_buf,
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
                            unary_op =
                                eval!(UnaryOp::parse_from_pair, warnings, errors, pair, None);
                        }
                        Rule::var_name_ident => {
                            name = Some(eval!(
                                Ident::parse_from_pair,
                                warnings,
                                errors,
                                pair,
                                Ident {
                                    primary_name: "error parsing var name",
                                    span: span.clone()
                                }
                            ));
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
                let array_exps = expr.into_inner();
                let mut contents = Vec::new();
                for expr in array_exps {
                    contents.push(eval!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        expr,
                        Expression::Unit { span: span.clone() }
                    ));
                }
                Expression::Array { contents, span }
            }
            Rule::match_expression => {
                let mut expr_iter = expr.into_inner();
                let primary_expression = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    expr_iter.next().unwrap(),
                    Expression::Unit { span: span.clone() }
                );
                let primary_expression = Box::new(primary_expression);
                let mut branches = Vec::new();
                for exp in expr_iter {
                    let res = eval!(
                        MatchBranch::parse_from_pair,
                        warnings,
                        errors,
                        exp,
                        MatchBranch {
                            condition: MatchCondition::CatchAll,
                            result: Expression::Unit { span: span.clone() },
                            span: span.clone()
                        }
                    );
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
                let struct_name = expr_iter.next().unwrap();
                let struct_name = eval!(
                    Ident::parse_from_pair,
                    warnings,
                    errors,
                    struct_name,
                    return err(warnings, errors)
                );
                let fields = expr_iter.next().unwrap().into_inner().collect::<Vec<_>>();
                let mut fields_buf = Vec::new();
                for i in (0..fields.len()).step_by(2) {
                    let name = eval!(
                        Ident::parse_from_pair,
                        warnings,
                        errors,
                        fields[i],
                        return err(warnings, errors)
                    );
                    let span = fields[i].as_span();
                    let value = eval!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        fields[i + 1].clone(),
                        Expression::Unit { span: span.clone() }
                    );
                    fields_buf.push(StructExpressionField { name, value, span });
                }

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
                    errors,
                    expr.clone().into_inner().next().unwrap(),
                    Expression::Unit {
                        span: expr.as_span()
                    }
                );
                Expression::ParenthesizedExpression {
                    inner: Box::new(expr),
                    span,
                }
            }
            Rule::code_block => {
                let whole_block_span = expr.as_span();
                let expr = eval!(
                    crate::CodeBlock::parse_from_pair,
                    warnings,
                    errors,
                    expr,
                    crate::CodeBlock {
                        contents: Vec::new(),
                        whole_block_span,
                        scope: Default::default()
                    }
                );
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
                let condition = Box::new(eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    condition_pair,
                    Expression::Unit { span: span.clone() }
                ));
                let then = Box::new(eval!(
                    Expression::parse_from_pair_inner,
                    warnings,
                    errors,
                    then_pair,
                    Expression::Unit { span: span.clone() }
                ));
                let r#else = match else_pair {
                    Some(else_pair) => Some(Box::new(eval!(
                        Expression::parse_from_pair_inner,
                        warnings,
                        errors,
                        else_pair,
                        Expression::Unit { span: span.clone() }
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
            Rule::asm_expression => {
                let whole_block_span = expr.as_span();
                let asm = eval!(
                    AsmExpression::parse_from_pair,
                    warnings,
                    errors,
                    expr,
                    return err(warnings, errors)
                );
                Expression::AsmExpression {
                    asm,
                    span: whole_block_span,
                }
            }
            Rule::method_exp => {
                let whole_exp_span = expr.as_span();
                let mut parts = expr.into_inner();
                let subfield_exp = parts.next().unwrap();
                assert_eq!(subfield_exp.as_rule(), Rule::subfield_exp);
                // remove the last field from the subfield exp, since it is the method name
                // the different parts of the exp
                // e.g.
                // if the method_exp is a.b.c.add()
                // then these parts are
                // ["a", "b", "c", "add"]
                let mut name_parts = subfield_exp.into_inner().collect::<Vec<_>>();
                let method_name = eval!(
                    CallPath::parse_from_pair,
                    warnings,
                    errors,
                    name_parts.pop().unwrap(),
                    return err(warnings, errors)
                );
                let function_arguments = parts
                    .next()
                    .map(|x| x.into_inner().collect::<Vec<_>>())
                    .unwrap_or_else(|| vec![]);
                let mut arguments_buf = VecDeque::new();
                for argument in function_arguments {
                    let arg = eval!(
                        Expression::parse_from_pair_inner,
                        warnings,
                        errors,
                        argument,
                        Expression::Unit {
                            span: argument.as_span()
                        }
                    );
                    arguments_buf.push_back(arg);
                }
                let mut name_parts_buf = Vec::new();
                for name_part in name_parts {
                    let name = eval!(
                        Ident::parse_from_pair,
                        warnings,
                        errors,
                        name_part,
                        continue
                    );
                    name_parts_buf.push(name);
                }

                if name_parts_buf.len() == 1 {
                    // then the first argument is a variable expression
                    arguments_buf.push_front(Expression::VariableExpression {
                        unary_op: None, // TODO
                        name: name_parts_buf[0].clone(),
                        span: name_parts_buf[0].clone().span,
                    });
                } else {
                    // then it is a subfield expression
                    // then the first argument is a variable expression
                    arguments_buf.push_front(Expression::SubfieldExpression {
                        name_parts: name_parts_buf.clone(),
                        unary_op: None, // TODO
                        span: name_parts_buf
                            .clone()
                            .into_iter()
                            .fold(name_parts_buf[0].clone().span, |acc, this| {
                                join_spans(acc, this.span)
                            }),
                    });
                }
                Expression::MethodApplication {
                    subfield_exp: vec![],
                    method_name,
                    arguments: arguments_buf.into_iter().collect(),
                    span: whole_exp_span,
                }
            }
            Rule::subfield_exp => eval!(
                subfield_from_pair,
                warnings,
                errors,
                expr,
                return err(warnings, errors)
            ),
            Rule::delineated_path => {
                // this is either an enum expression or looking something
                // up in libraries
                let span = expr.as_span();
                let mut parts = expr.into_inner();
                let path_component = parts.next().unwrap();
                let instantiator = parts.next();
                let path = eval!(
                    CallPath::parse_from_pair,
                    warnings,
                    errors,
                    path_component,
                    return err(warnings, errors)
                );

                let instantiator = if let Some(inst) = instantiator {
                    Some(Box::new(eval!(
                        Expression::parse_from_pair_inner,
                        warnings,
                        errors,
                        inst.into_inner().next().unwrap(),
                        return err(warnings, errors)
                    )))
                } else {
                    None
                };

                // if there is an expression in parenthesis, that is the instantiator.

                Expression::DelineatedPath {
                    call_path: path,
                    instantiator,
                    span,
                    // Eventually, when we support generic enums, we want to be able to parse type
                    // arguments on the enum name and throw them in here. TODO
                    type_arguments: vec![],
                }
            }
            Rule::unit => Expression::Unit {
                span: expr.as_span(),
            },
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(a, expr.as_span()));
                // construct unit expression for error recovery
                Expression::Unit {
                    span: expr.as_span(),
                }
            }
        };
        ok(parsed, warnings, errors)
    }
}

#[derive(Debug, Clone)]
pub struct MatchBranch<'sc> {
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
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let span = pair.as_span();
        let mut branch = pair.clone().into_inner();
        let condition = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    pair.as_span(),
                ));
                return err(warnings, errors);
            }
        };
        let condition = match condition.into_inner().next() {
            Some(e) => {
                let expr = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    e,
                    Expression::Unit { span: e.as_span() }
                );
                MatchCondition::Expression(expr)
            }
            // the "_" case
            None => MatchCondition::CatchAll,
        };
        let result = match branch.next() {
            Some(o) => o,
            None => {
                errors.push(CompileError::Internal(
                    "Unexpected empty iterator in match branch parsing.",
                    pair.as_span(),
                ));
                return err(warnings, errors);
            }
        };
        let result = match result.as_rule() {
            Rule::expr => eval!(
                Expression::parse_from_pair,
                warnings,
                errors,
                result,
                Expression::Unit {
                    span: result.as_span()
                }
            ),
            Rule::code_block => {
                let span = result.as_span();
                Expression::CodeBlock {
                    contents: eval!(
                        CodeBlock::parse_from_pair,
                        warnings,
                        errors,
                        result,
                        CodeBlock {
                            contents: Vec::new(),
                            whole_block_span: span.clone(),
                            scope: HashMap::default()
                        }
                    ),
                    span,
                }
            }
            _ => unreachable!(),
        };
        ok(
            MatchBranch {
                condition,
                result,
                span,
            },
            warnings,
            errors,
        )
    }
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Ref,
    Deref,
}

impl UnaryOp {
    fn parse_from_pair<'sc>(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Option<Self>> {
        use UnaryOp::*;
        match pair.as_str() {
            "!" => ok(Some(Not), Vec::new(), Vec::new()),
            "ref" => ok(Some(Ref), Vec::new(), Vec::new()),
            "deref" => ok(Some(Deref), Vec::new(), Vec::new()),
            _ => {
                let errors = vec![CompileError::Internal(
                    "Attempted to parse unary op from invalid op string.",
                    pair.as_span(),
                )];
                return err(Vec::new(), errors);
            }
        }
    }
}

fn parse_op<'sc>(op: Pair<'sc, Rule>) -> CompileResult<'sc, Op> {
    use OpVariant::*;
    let mut errors = Vec::new();
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
        ">" => GreaterThan,
        "<" => LessThan,
        a => {
            errors.push(CompileError::ExpectedOp {
                op: a,
                span: op.as_span(),
            });
            return err(Vec::new(), errors);
        }
    };
    ok(
        Op {
            span: op.as_span(),
            op_variant,
        },
        Vec::new(),
        errors,
    )
}

#[derive(Debug)]
struct Op<'sc> {
    span: Span<'sc>,
    op_variant: OpVariant,
}

impl<'sc> Op<'sc> {
    fn to_var_name(&self) -> Ident<'sc> {
        Ident {
            primary_name: self.op_variant.as_str(),
            span: self.span.clone(),
            // TODO this should be a method exp not a var name
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
    GreaterThan,
    LessThan,
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
            GreaterThan => "greater_than",
            LessThan => "less_than",
        }
    }
    fn precedence(&self) -> usize {
        use OpVariant::*;
        // a higher number means the operation has higher precedence
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
            GreaterThan => 0,
            LessThan => 0,
        }
    }
}

fn arrange_by_order_of_operations<'sc>(
    expressions: Vec<Either<Op<'sc>, Expression<'sc>>>,
    debug_span: Span<'sc>,
) -> CompileResult<'sc, Expression<'sc>> {
    let mut errors = Vec::new();
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
                        errors.push(CompileError::Internal(
                            "Prematurely empty expression stack for left hand side.",
                            debug_span,
                        ));
                        return err(warnings, errors);
                    }
                    if rhs.is_none() {
                        errors.push(CompileError::Internal(
                            "Prematurely empty expression stack for right hand side.",
                            debug_span,
                        ));
                        return err(warnings, errors);
                    }
                    let lhs = lhs.unwrap();
                    let rhs = rhs.unwrap();
                    expression_stack.push(Expression::FunctionApplication {
                        name: CallPath {
                            prefixes: vec![
                                Ident {
                                    primary_name: "std".into(),
                                    span: new_op.span.clone(),
                                },
                                Ident {
                                    primary_name: "ops".into(),
                                    span: new_op.span.clone(),
                                },
                            ],
                            suffix: new_op.to_var_name(),
                        },
                        arguments: vec![lhs, rhs],
                        span: debug_span.clone(),
                    });
                }
                op_stack.push(op)
            }
            Either::Right(expr) => expression_stack.push(expr),
        }
    }

    while let Some(op) = op_stack.pop() {
        let rhs = expression_stack.pop();
        let lhs = expression_stack.pop();

        if lhs.is_none() {
            errors.push(CompileError::Internal(
                "Prematurely empty expression stack for left hand side.",
                debug_span,
            ));
            return err(warnings, errors);
        }
        if rhs.is_none() {
            errors.push(CompileError::Internal(
                "Prematurely empty expression stack for right hand side.",
                debug_span,
            ));
            return err(warnings, errors);
        }

        let lhs = lhs.unwrap();
        let rhs = rhs.unwrap();

        expression_stack.push(Expression::MethodApplication {
            method_name: CallPath {
                prefixes: vec![
                    Ident {
                        primary_name: "std".into(),
                        span: op.span.clone(),
                    },
                    Ident {
                        primary_name: "ops".into(),
                        span: op.span.clone(),
                    },
                ],
                suffix: op.to_var_name(),
            },
            subfield_exp: vec![],
            arguments: vec![lhs.clone(), rhs.clone()],
            span: join_spans(join_spans(lhs.span(), op.span.clone()), rhs.span()),
        });
    }

    if expression_stack.len() != 1 {
        errors.push(CompileError::Internal(
            "Invalid expression stack length",
            debug_span,
        ));
        return err(warnings, errors);
    }

    ok(expression_stack[0].clone(), warnings, errors)
}
fn subfield_from_pair<'sc>(expr: Pair<'sc, Rule>) -> CompileResult<'sc, Expression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let span = expr.as_span();
    let iter = expr.into_inner();
    let mut buf = vec![];
    for part in iter {
        buf.push(eval!(
            Ident::parse_from_pair,
            warnings,
            errors,
            part,
            continue
        ));
    }
    ok(
        Expression::SubfieldExpression {
            span,
            name_parts: buf,
            unary_op: None, // TODO: support unary operators before subfield expressions
        },
        warnings,
        errors,
    )
}
