use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, literal::handle_parse_int_error, CallPath, Literal},
    parser::Rule,
    type_engine::{IntegerBits, TypeInfo},
    AstNode, AstNodeContent, CodeBlock, Declaration, TypeArgument, VariableDeclaration,
};

use sway_types::{ident::Ident, Span};

use either::Either;
use pest;
use pest::iterators::Pair;
use std::collections::VecDeque;

mod asm;
mod match_branch;
mod match_condition;
mod matcher;
mod method_name;
mod scrutinee;
mod unary_op;
pub(crate) use asm::*;
pub(crate) use match_branch::MatchBranch;
pub(crate) use match_condition::CatchAll;
pub(crate) use match_condition::MatchCondition;
use matcher::matcher;
pub use method_name::MethodName;
pub(crate) use scrutinee::{Scrutinee, StructScrutineeField};
pub(crate) use unary_op::UnaryOp;

/// Represents a parsed, but not yet type checked, [Expression](https://en.wikipedia.org/wiki/Expression_(computer_science)).
#[derive(Debug, Clone)]
pub enum Expression {
    Literal {
        value: Literal,
        span: Span,
    },
    FunctionApplication {
        name: CallPath,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    LazyOperator {
        op: LazyOp,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        span: Span,
    },
    VariableExpression {
        name: Ident,
        span: Span,
    },
    Tuple {
        fields: Vec<Expression>,
        span: Span,
    },
    TupleIndex {
        prefix: Box<Expression>,
        index: usize,
        index_span: Span,
        span: Span,
    },
    Array {
        contents: Vec<Expression>,
        span: Span,
    },
    StructExpression {
        struct_name: CallPath,
        type_arguments: Vec<TypeArgument>,
        fields: Vec<StructExpressionField>,
        span: Span,
    },
    CodeBlock {
        contents: CodeBlock,
        span: Span,
    },
    IfExp {
        condition: Box<Expression>,
        then: Box<Expression>,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    MatchExp {
        if_exp: Box<Expression>,
        cases_covered: Vec<MatchCondition>,
        span: Span,
    },
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span,
        asm: AsmExpression,
    },
    MethodApplication {
        method_name: MethodName,
        contract_call_params: Vec<StructExpressionField>,
        arguments: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        span: Span,
    },
    /// A _subfield expression_ is anything of the form:
    /// ```ignore
    /// <ident>.<ident>
    /// ```
    ///
    SubfieldExpression {
        prefix: Box<Expression>,
        span: Span,
        field_to_access: Ident,
    },
    /// A _delineated path_ is anything of the form:
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
        call_path: CallPath,
        args: Vec<Expression>,
        span: Span,
        type_arguments: Vec<TypeArgument>,
    },
    /// A cast of a hash to an ABI for calling a contract.
    AbiCast {
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
    },
    ArrayIndex {
        prefix: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    /// This variant serves as a stand-in for parsing-level match expression desugaring.
    /// Because types cannot be known at parsing-time, a desugared struct or enum gets
    /// special cased into this variant. During type checking, this variant is removed
    /// as is replaced with the corresponding field or argument access (given that the
    /// expression inside of the delayed resolution has the appropriate struct or enum
    /// type)
    DelayedMatchTypeResolution {
        variant: DelayedResolutionVariant,
        span: Span,
    },
    StorageAccess {
        field_names: Vec<Ident>,
        span: Span,
    },
    IfLet {
        scrutinee: Scrutinee,
        expr: Box<Expression>,
        then: CodeBlock,
        r#else: Option<Box<Expression>>,
        span: Span,
    },
    SizeOfVal {
        exp: Box<Expression>,
        span: Span,
    },
    BuiltinGetTypeProperty {
        builtin: BuiltinProperty,
        type_name: TypeInfo,
        type_span: Span,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinProperty {
    SizeOfType,
    IsRefType,
}

#[derive(Debug, Clone)]
pub enum DelayedResolutionVariant {
    StructField(DelayedStructFieldResolution),
    EnumVariant(DelayedEnumVariantResolution),
    TupleVariant(DelayedTupleVariantResolution),
}

/// During type checking, this gets replaced with struct field access.
#[derive(Debug, Clone)]
pub struct DelayedStructFieldResolution {
    pub exp: Box<Expression>,
    pub struct_name: Ident,
    pub field: Ident,
}

/// During type checking, this gets replaced with an if let, maybe, although that's not yet been
/// implemented.
#[derive(Debug, Clone)]
pub struct DelayedEnumVariantResolution {
    pub exp: Box<Expression>,
    pub call_path: CallPath,
    pub arg_num: usize,
}

/// During type checking, this gets replaced with tuple arg access.
#[derive(Debug, Clone)]
pub struct DelayedTupleVariantResolution {
    pub exp: Box<Expression>,
    pub elem_num: usize,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum LazyOp {
    And,
    Or,
}

impl LazyOp {
    fn from(op_variant: OpVariant) -> Self {
        match op_variant {
            OpVariant::And => Self::And,
            OpVariant::Or => Self::Or,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructExpressionField {
    pub name: Ident,
    pub value: Expression,
    pub(crate) span: Span,
}

pub(crate) fn error_recovery_exp(span: Span) -> Expression {
    Expression::Tuple {
        fields: vec![],
        span,
    }
}

impl Expression {
    pub(crate) fn core_ops_eq(arguments: Vec<Expression>, span: Span) -> Expression {
        Expression::MethodApplication {
            method_name: MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: Op {
                        op_variant: OpVariant::Equals,
                        span: span.clone(),
                    }
                    .to_var_name(),
                    is_absolute: true,
                },
                type_name: None,
                type_name_span: None,
            },
            contract_call_params: vec![],
            arguments,
            type_arguments: vec![],
            span,
        }
    }

    pub(crate) fn core_ops(op: Op, arguments: Vec<Expression>, span: Span) -> Expression {
        Expression::MethodApplication {
            method_name: MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: op.to_var_name(),
                    is_absolute: true,
                },
                type_name: None,
                type_name_span: None,
            },
            contract_call_params: vec![],
            arguments,
            type_arguments: vec![],
            span,
        }
    }

    pub(crate) fn span(&self) -> Span {
        use Expression::*;
        (match self {
            Literal { span, .. } => span,
            FunctionApplication { span, .. } => span,
            LazyOperator { span, .. } => span,
            VariableExpression { span, .. } => span,
            Tuple { span, .. } => span,
            TupleIndex { span, .. } => span,
            Array { span, .. } => span,
            StructExpression { span, .. } => span,
            CodeBlock { span, .. } => span,
            IfExp { span, .. } => span,
            MatchExp { span, .. } => span,
            AsmExpression { span, .. } => span,
            MethodApplication { span, .. } => span,
            SubfieldExpression { span, .. } => span,
            DelineatedPath { span, .. } => span,
            AbiCast { span, .. } => span,
            ArrayIndex { span, .. } => span,
            DelayedMatchTypeResolution { span, .. } => span,
            StorageAccess { span, .. } => span,
            IfLet { span, .. } => span,
            SizeOfVal { span, .. } => span,
            BuiltinGetTypeProperty { span, .. } => span,
        })
        .clone()
    }

    pub(crate) fn parse_from_pair(
        expr: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Self>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_for_debug = expr.clone();
        let mut expr_iter = expr.into_inner();
        // first expr is always here
        let first_expr = expr_iter.next().unwrap();
        let first_expr_result = check!(
            Expression::parse_from_pair_inner(first_expr.clone(), config),
            ParserLifter::empty(error_recovery_exp(Span::from_pest(
                first_expr.as_span(),
                path.clone(),
            ))),
            warnings,
            errors
        );
        let mut expr_result_or_op_buf: Vec<Either<Op, ParserLifter<Expression>>> =
            vec![Either::Right(first_expr_result.clone())];
        // sometimes exprs are followed by ops in the same expr
        while let Some(op) = expr_iter.next() {
            let op_str = op.as_str().to_string();
            let op_span = Span::from_pest(op.as_span(), path.clone());

            let op = check!(
                parse_op(op, config),
                return err(warnings, errors),
                warnings,
                errors
            );

            // an op is necessarily followed by an expression
            let next_expr_result = match expr_iter.next() {
                Some(o) => check!(
                    Expression::parse_from_pair_inner(o.clone(), config),
                    ParserLifter::empty(error_recovery_exp(Span::from_pest(
                        o.as_span(),
                        path.clone()
                    ))),
                    warnings,
                    errors
                ),
                None => {
                    errors.push(CompileError::ExpectedExprAfterOp {
                        op: op_str,
                        span: Span::from_pest(expr_for_debug.as_span(), path.clone()),
                    });
                    ParserLifter::empty(error_recovery_exp(op_span))
                }
            };
            // pushing these into a vec in this manner so we can re-associate according to order of
            // operations later
            expr_result_or_op_buf.push(Either::Left(op));
            expr_result_or_op_buf.push(Either::Right(next_expr_result));
            /*
             * TODO
             * strategy: keep parsing until we have all of the op expressions
             * re-associate the expr tree with operator precedence
             */
        }
        if expr_result_or_op_buf.len() == 1 {
            ok(first_expr_result, warnings, errors)
        } else {
            let expr_result = arrange_by_order_of_operations(
                expr_result_or_op_buf,
                Span::from_pest(expr_for_debug.as_span(), path.clone()),
            )
            .unwrap_or_else(&mut warnings, &mut errors, || {
                ParserLifter::empty(error_recovery_exp(Span::from_pest(
                    expr_for_debug.as_span(),
                    path.clone(),
                )))
            });
            ok(expr_result, warnings, errors)
        }
    }

    pub(crate) fn parse_from_pair_inner(
        expr: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParserLifter<Self>> {
        let path = config.map(|c| c.path());
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = Span::from_pest(expr.as_span(), path.clone());
        let parsed_result = match expr.as_rule() {
            Rule::literal_value => Literal::parse_from_pair(expr, config)
                .map(|(value, span)| ParserLifter::empty(Expression::Literal { value, span }))
                .unwrap_or_else(&mut warnings, &mut errors, || {
                    ParserLifter::empty(error_recovery_exp(span))
                }),
            Rule::func_app => {
                let span = Span::from_pest(expr.as_span(), path.clone());
                let mut func_app_parts = expr.into_inner();
                let first_part = func_app_parts.next().unwrap();
                assert!(first_part.as_rule() == Rule::call_path);
                let name = check!(
                    CallPath::parse_from_pair(first_part, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (arguments, type_args_with_path) = {
                    let maybe_type_args = func_app_parts.next().unwrap();
                    match maybe_type_args.as_rule() {
                        Rule::type_args_with_path => {
                            (func_app_parts.next().unwrap(), Some(maybe_type_args))
                        }
                        Rule::fn_args => (maybe_type_args, None),
                        _ => unreachable!(),
                    }
                };
                let mut arguments_buf = Vec::new();
                for argument in arguments.into_inner() {
                    let arg = check!(
                        Expression::parse_from_pair(argument.clone(), config),
                        ParserLifter::empty(error_recovery_exp(Span::from_pest(
                            argument.as_span(),
                            path.clone()
                        ))),
                        warnings,
                        errors
                    );
                    arguments_buf.push(arg);
                }

                let maybe_type_args = type_args_with_path
                    .map(|x| {
                        x.into_inner()
                            .nth(1)
                            .expect("guaranteed by grammar")
                            .into_inner()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(Vec::new);
                let mut type_arguments = vec![];
                for type_arg in maybe_type_args.into_iter() {
                    type_arguments.push(check!(
                        TypeArgument::parse_from_pair(type_arg, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }

                let var_decls = arguments_buf
                    .iter()
                    .flat_map(|x| x.var_decls.clone())
                    .collect::<Vec<_>>();
                let arguments_buf = arguments_buf
                    .into_iter()
                    .map(|x| x.value)
                    .collect::<Vec<_>>();
                let exp = Expression::FunctionApplication {
                    name,
                    arguments: arguments_buf,
                    span,
                    type_arguments,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::var_exp => {
                // this means that this is something like `!`, `ref`, or `deref` and the next
                // token is the actual expr value
                let mut name = None;
                for pair in expr.into_inner() {
                    match pair.as_rule() {
                        Rule::var_name_ident => {
                            name = Some(check!(
                                ident::parse_from_pair(pair, config),
                                Ident::new_with_override("error parsing var name", span.clone()),
                                warnings,
                                errors
                            ));
                        }
                        a => unreachable!("what is this? {:?} {}", a, pair.as_str()),
                    }
                }
                // this is non-optional and part of the parse rule so it won't fail
                let name = name.unwrap();
                let exp = Expression::VariableExpression { name, span };
                ParserLifter::empty(exp)
            }
            Rule::var_name_ident => {
                let name = check!(
                    ident::parse_from_pair(expr, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ParserLifter::empty(Expression::VariableExpression { name, span })
            }
            Rule::array_exp => match expr.into_inner().next() {
                None => ParserLifter::empty(Expression::Array {
                    contents: Vec::new(),
                    span,
                }),
                Some(array_elems) => check!(
                    parse_array_elems(array_elems, config),
                    ParserLifter::empty(error_recovery_exp(span)),
                    warnings,
                    errors
                ),
            },
            Rule::match_expression => {
                let mut expr_iter = expr.into_inner();
                let primary_expression_result = check!(
                    Expression::parse_from_pair(expr_iter.next().unwrap(), config),
                    ParserLifter::empty(error_recovery_exp(span.clone())),
                    warnings,
                    errors
                );
                let mut branches = Vec::new();
                for exp in expr_iter {
                    let res = check!(
                        MatchBranch::parse_from_pair(exp, config),
                        MatchBranch {
                            condition: MatchCondition::CatchAll(CatchAll { span: span.clone() }),
                            result: Expression::Tuple {
                                fields: vec![],
                                span: span.clone(),
                            },
                            span: span.clone()
                        },
                        warnings,
                        errors
                    );
                    branches.push(res);
                }
                let primary_expression = primary_expression_result.value;
                let (if_exp, var_decl_name, cases_covered) = check!(
                    desugar_match_expression(&primary_expression, branches, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut var_decls = primary_expression_result.var_decls;
                var_decls.push(VariableDeclaration {
                    name: var_decl_name,
                    type_ascription: TypeInfo::Unknown,
                    type_ascription_span: None,
                    is_mutable: false,
                    body: primary_expression,
                });
                let exp = Expression::MatchExp {
                    if_exp: Box::new(if_exp),
                    cases_covered,
                    span,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::struct_expression => {
                let mut expr_iter = expr.into_inner().peekable();
                let struct_name = expr_iter.next().unwrap();
                let struct_name = check!(
                    CallPath::parse_from_pair(struct_name, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let type_arguments = match expr_iter.peek() {
                    Some(pair) if pair.as_rule() == Rule::type_args_with_path => check!(
                        TypeArgument::parse_arguments_from_pair(
                            expr_iter
                                .next()
                                .unwrap()
                                .into_inner()
                                .nth(1)
                                .expect("guaranteed by grammar"),
                            config
                        ),
                        vec!(),
                        warnings,
                        errors
                    ),
                    _ => vec![],
                };
                let fields = expr_iter.next().unwrap().into_inner().collect::<Vec<_>>();
                let mut fields_buf = Vec::new();
                let mut var_decls = vec![];
                for i in (0..fields.len()).step_by(2) {
                    let name = check!(
                        ident::parse_from_pair(fields[i].clone(), config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    let span = Span::from_pest(fields[i].as_span(), path.clone());
                    let mut value_result = check!(
                        Expression::parse_from_pair(fields[i + 1].clone(), config),
                        ParserLifter::empty(error_recovery_exp(span.clone())),
                        warnings,
                        errors
                    );
                    fields_buf.push(StructExpressionField {
                        name,
                        value: value_result.value,
                        span,
                    });
                    var_decls.append(&mut value_result.var_decls);
                }

                let exp = Expression::StructExpression {
                    struct_name,
                    type_arguments,
                    fields: fields_buf,
                    span,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::parenthesized_expression => {
                check!(
                    Expression::parse_from_pair(expr.clone().into_inner().next().unwrap(), config),
                    ParserLifter::empty(error_recovery_exp(Span::from_pest(expr.as_span(), path,))),
                    warnings,
                    errors
                )
            }
            Rule::code_block => {
                let whole_block_span = Span::from_pest(expr.as_span(), path);
                let expr = check!(
                    CodeBlock::parse_from_pair(expr, config),
                    CodeBlock {
                        contents: Vec::new(),
                        whole_block_span,
                    },
                    warnings,
                    errors
                );
                let exp = Expression::CodeBlock {
                    contents: expr,
                    span,
                };
                // this assumes that all of the var_decls will be injected into the code block
                ParserLifter::empty(exp)
            }
            Rule::if_exp => {
                let span = Span::from_pest(expr.as_span(), path);
                let mut if_exp_pairs = expr.into_inner();
                let condition_pair = if_exp_pairs.next().unwrap();
                let then_pair = if_exp_pairs.next().unwrap();
                let else_pair = if_exp_pairs.next();
                let condition_result = check!(
                    Expression::parse_from_pair(condition_pair, config),
                    ParserLifter::empty(error_recovery_exp(span.clone())),
                    warnings,
                    errors
                );
                let mut then_result = check!(
                    Expression::parse_from_pair_inner(then_pair, config),
                    ParserLifter::empty(error_recovery_exp(span.clone())),
                    warnings,
                    errors
                );
                let r#else_result = else_pair.map(|else_pair| {
                    check!(
                        Expression::parse_from_pair_inner(else_pair, config),
                        ParserLifter::empty(error_recovery_exp(span.clone())),
                        warnings,
                        errors
                    )
                });
                let mut var_decls = condition_result.var_decls;
                var_decls.append(&mut then_result.var_decls);
                let mut r#else_result_decls = match r#else_result.clone() {
                    Some(r#else_result) => r#else_result.var_decls,
                    None => vec![],
                };
                var_decls.append(&mut r#else_result_decls);
                let exp = Expression::IfExp {
                    condition: Box::new(condition_result.value),
                    then: Box::new(then_result.value),
                    r#else: r#else_result.map(|x| Box::new(x.value)),
                    span,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::asm_expression => {
                let whole_block_span = Span::from_pest(expr.as_span(), path);
                let asm_result = check!(
                    AsmExpression::parse_from_pair(expr, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let exp = Expression::AsmExpression {
                    asm: asm_result.value,
                    span: whole_block_span,
                };
                ParserLifter {
                    var_decls: asm_result.var_decls,
                    value: exp,
                }
            }
            Rule::method_exp => {
                let whole_exp_span = Span::from_pest(expr.as_span(), path.clone());
                let mut parts = expr.into_inner();
                let pair = parts.next().unwrap();
                match pair.as_rule() {
                    Rule::subfield_exp => {
                        let mut pair = pair.into_inner();
                        let mut name_parts = pair
                            .next()
                            .expect("Guaranteed by grammar.")
                            .into_inner()
                            .collect::<Vec<_>>();
                        let contract_call_params =
                            match pair.peek().expect("Guaranteed by grammar").as_rule() {
                                Rule::contract_call_params => pair.next(),
                                _ => None,
                            };

                        let mut var_decls_buf = Vec::new();
                        let mut fields_buf = Vec::new();
                        if let Some(params) = contract_call_params {
                            let fields = params
                                .into_inner()
                                .next()
                                .unwrap()
                                .into_inner()
                                .collect::<Vec<_>>();
                            for i in (0..fields.len()).step_by(2) {
                                let name = check!(
                                    ident::parse_from_pair(fields[i].clone(), config),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                );
                                let span = Span::from_pest(fields[i].as_span(), path.clone());
                                let ParserLifter {
                                    value,
                                    mut var_decls,
                                } = check!(
                                    Expression::parse_from_pair(fields[i + 1].clone(), config),
                                    ParserLifter::empty(error_recovery_exp(span.clone())),
                                    warnings,
                                    errors
                                );
                                var_decls_buf.append(&mut var_decls);
                                fields_buf.push(StructExpressionField { name, value, span });
                            }
                        }

                        let function_arguments =
                            pair.next().expect("Guaranteed by grammar").into_inner();
                        // remove the last field from the subfield exp, since it is the method name
                        // the different parts of the exp
                        // e.g.
                        // if the method_exp is a.b.c.add()
                        // then these parts are
                        //
                        // ["a", "b", "c", "add"]
                        let method_name = check!(
                            ident::parse_from_pair(name_parts.pop().unwrap(), config),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let mut arguments_buf = VecDeque::new();
                        for argument in function_arguments {
                            let ParserLifter {
                                value,
                                mut var_decls,
                            } = check!(
                                Expression::parse_from_pair(argument.clone(), config),
                                ParserLifter::empty(error_recovery_exp(Span::from_pest(
                                    argument.as_span(),
                                    path.clone()
                                ))),
                                warnings,
                                errors
                            );
                            var_decls_buf.append(&mut var_decls);
                            arguments_buf.push_back(value);
                        }
                        // the first thing is either an exp or a var, everything subsequent must be
                        // a field
                        let mut name_parts = name_parts.into_iter();
                        let ParserLifter {
                            value,
                            mut var_decls,
                        } = check!(
                            parse_subfield_path(
                                name_parts.next().expect("guaranteed by grammar"),
                                config
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        let mut expr = value;
                        var_decls_buf.append(&mut var_decls);

                        for name_part in name_parts {
                            expr = Expression::SubfieldExpression {
                                prefix: Box::new(expr.clone()),
                                span: Span::from_pest(name_part.as_span(), path.clone()),
                                field_to_access: check!(
                                    ident::parse_from_pair(name_part, config),
                                    continue,
                                    warnings,
                                    errors
                                ),
                            };
                        }

                        arguments_buf.push_front(expr);
                        let exp = Expression::MethodApplication {
                            method_name: MethodName::FromModule { method_name },
                            contract_call_params: fields_buf,
                            arguments: arguments_buf.into_iter().collect(),
                            type_arguments: vec![],
                            span: whole_exp_span,
                        };
                        ParserLifter {
                            var_decls: var_decls_buf,
                            value: exp,
                        }
                    }
                    Rule::fully_qualified_method => {
                        let mut call_path = None;
                        let mut type_name = None;
                        let mut method_name = None;
                        let mut type_args_with_path = None;
                        let mut arguments = None;
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::path_separator => (),
                                Rule::call_path => {
                                    call_path = Some(check!(
                                        CallPath::parse_from_pair(pair, config),
                                        continue,
                                        warnings,
                                        errors
                                    ));
                                }
                                Rule::ident => {
                                    type_name = Some(pair);
                                }
                                Rule::call_item => {
                                    method_name = Some(pair);
                                }
                                Rule::type_args_with_path => {
                                    type_args_with_path = Some(pair);
                                }
                                Rule::fn_args => {
                                    arguments = Some(pair);
                                }
                                a => unreachable!("guaranteed by grammar: {:?}", a),
                            }
                        }

                        let (type_name, type_name_span) = match type_name {
                            Some(type_name) => {
                                let type_name_span =
                                    Span::from_pest(type_name.as_span(), path.clone());
                                (
                                    TypeInfo::pair_as_str_to_type_info(type_name, config),
                                    type_name_span,
                                )
                            }
                            None => {
                                return err(warnings, errors);
                            }
                        };

                        let type_arguments = match type_args_with_path {
                            Some(type_args_with_path) => check!(
                                TypeArgument::parse_arguments_from_pair(
                                    type_args_with_path
                                        .into_inner()
                                        .nth(1)
                                        .expect("guaranteed by grammar"),
                                    config
                                ),
                                vec!(),
                                warnings,
                                errors
                            ),
                            None => vec![],
                        };

                        let (call_path, is_absolute) = match call_path {
                            Some(call_path) => {
                                let mut call_path_buf = call_path.prefixes;
                                call_path_buf.push(call_path.suffix);
                                (call_path_buf, call_path.is_absolute)
                            }
                            None => (vec![], false),
                        };
                        let method_name = MethodName::FromType {
                            call_path: CallPath {
                                prefixes: call_path,
                                suffix: check!(
                                    ident::parse_from_pair(
                                        method_name.expect("guaranteed by grammar"),
                                        config
                                    ),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                ),
                                is_absolute,
                            },
                            type_name: Some(check!(
                                type_name,
                                TypeInfo::ErrorRecovery,
                                warnings,
                                errors
                            )),
                            type_name_span: Some(type_name_span),
                        };

                        let mut argument_results_buf = vec![];
                        // evaluate  the arguments passed in to the method
                        if let Some(arguments) = arguments {
                            for argument in arguments.into_inner() {
                                let arg_result = check!(
                                    Expression::parse_from_pair(argument.clone(), config),
                                    ParserLifter::empty(error_recovery_exp(Span::from_pest(
                                        argument.as_span(),
                                        path.clone()
                                    ))),
                                    warnings,
                                    errors
                                );
                                argument_results_buf.push(arg_result);
                            }
                        }

                        let var_decls = argument_results_buf
                            .iter()
                            .flat_map(|x| x.var_decls.clone())
                            .collect::<Vec<_>>();
                        let arguments_buf = argument_results_buf
                            .into_iter()
                            .map(|x| x.value)
                            .collect::<Vec<_>>();
                        let exp = Expression::MethodApplication {
                            method_name,
                            contract_call_params: vec![],
                            arguments: arguments_buf,
                            type_arguments,
                            span: whole_exp_span,
                        };
                        ParserLifter {
                            var_decls,
                            value: exp,
                        }
                    }
                    a => unreachable!("{:?}", a),
                }
            }
            Rule::delineated_path => {
                // this is either an enum expression or looking something
                // up in libraries
                let span = Span::from_pest(expr.as_span(), path);
                let mut parts = expr.into_inner();
                let path_component = parts.next().unwrap();
                let (maybe_type_args, maybe_instantiator) = {
                    let part = parts.next();
                    match part.as_ref().map(|x| x.as_rule()) {
                        Some(Rule::fn_args) => (None, part),
                        Some(Rule::type_args_with_path) => {
                            let next_part = parts.next();
                            (part, next_part)
                        }
                        None => (None, None),
                        Some(_) => unreachable!("guaranteed by grammar"),
                    }
                };
                let path = check!(
                    CallPath::parse_from_pair(path_component, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let arg_results = if let Some(inst) = maybe_instantiator {
                    let mut buf = vec![];
                    for exp in inst.into_inner() {
                        let exp_result = check!(
                            Expression::parse_from_pair(exp, config),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        buf.push(exp_result);
                    }
                    buf
                } else {
                    vec![]
                };

                let maybe_type_args = maybe_type_args
                    .map(|x| {
                        x.into_inner()
                            .nth(1)
                            .expect("guaranteed by grammar")
                            .into_inner()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(Vec::new);
                let mut type_arguments = vec![];
                for arg in maybe_type_args {
                    type_arguments.push(check!(
                        TypeArgument::parse_from_pair(arg, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ));
                }

                let var_decls = arg_results
                    .iter()
                    .flat_map(|x| x.var_decls.clone())
                    .collect::<Vec<_>>();
                let args = arg_results.into_iter().map(|x| x.value).collect::<Vec<_>>();
                let exp = Expression::DelineatedPath {
                    call_path: path,
                    type_arguments,
                    args,
                    span,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::tuple_expr => {
                let fields = expr.into_inner().collect::<Vec<_>>();
                let mut field_results_buf = Vec::with_capacity(fields.len());
                for field in fields {
                    let value_result = check!(
                        Expression::parse_from_pair(field.clone(), config),
                        ParserLifter::empty(error_recovery_exp(span.clone())),
                        warnings,
                        errors
                    );
                    field_results_buf.push(value_result);
                }
                let var_decls = field_results_buf
                    .iter()
                    .flat_map(|x| x.var_decls.clone())
                    .collect::<Vec<_>>();
                let fields_buf = field_results_buf
                    .into_iter()
                    .map(|x| x.value)
                    .collect::<Vec<_>>();
                let exp = Expression::Tuple {
                    fields: fields_buf,
                    span,
                };
                ParserLifter {
                    var_decls,
                    value: exp,
                }
            }
            Rule::tuple_index => {
                let span = Span::from_pest(expr.as_span(), path.clone());
                let mut inner = expr.into_inner();
                let call_item = inner.next().expect("guarenteed by grammar");
                assert_eq!(call_item.as_rule(), Rule::call_item);
                let prefix_result = check!(
                    parse_call_item(call_item, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let the_integer = inner.next().expect("guarenteed by grammar");
                let the_integer_span = Span::from_pest(the_integer.as_span(), path.clone());
                let index: Result<usize, CompileError> =
                    the_integer.as_str().trim().parse().map_err(|e| {
                        handle_parse_int_error(
                            e,
                            TypeInfo::UnsignedInteger(IntegerBits::Eight),
                            the_integer.as_span(),
                            path.clone(),
                        )
                    });
                let index = match index {
                    Ok(index) => index,
                    Err(e) => {
                        errors.push(e);
                        return err(warnings, errors);
                    }
                };
                let exp = Expression::TupleIndex {
                    prefix: Box::new(prefix_result.value),
                    index,
                    index_span: the_integer_span,
                    span,
                };
                ParserLifter {
                    var_decls: prefix_result.var_decls,
                    value: exp,
                }
            }
            Rule::struct_field_access => {
                let inner = expr.into_inner().next().expect("guaranteed by grammar");
                assert_eq!(inner.as_rule(), Rule::subfield_path);

                let mut name_parts = inner.into_inner();
                let mut expr_result = check!(
                    parse_subfield_path(name_parts.next().expect("guaranteed by grammar"), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                for name_part in name_parts {
                    let new_expr = Expression::SubfieldExpression {
                        prefix: Box::new(expr_result.value.clone()),
                        span: Span::from_pest(name_part.as_span(), path.clone()),
                        field_to_access: check!(
                            ident::parse_from_pair(name_part, config),
                            continue,
                            warnings,
                            errors
                        ),
                    };
                    expr_result = ParserLifter {
                        var_decls: expr_result.var_decls,
                        value: new_expr,
                    };
                }

                expr_result
            }
            Rule::abi_cast => {
                let span = Span::from_pest(expr.as_span(), path);
                let mut iter = expr.into_inner();
                let _abi_keyword = iter.next();
                let abi_name = iter.next().expect("guaranteed by grammar");
                let abi_name = check!(
                    CallPath::parse_from_pair(abi_name, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let address = iter.next().expect("guaranteed by grammar");
                let address_result = check!(
                    Expression::parse_from_pair(address, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let exp = Expression::AbiCast {
                    span,
                    address: Box::new(address_result.value.clone()),
                    abi_name,
                };
                ParserLifter {
                    var_decls: address_result.var_decls,
                    value: exp,
                }
            }
            Rule::unary_op_expr => {
                check!(
                    convert_unary_to_fn_calls(expr, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            Rule::array_index => check!(
                parse_array_index(expr, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::storage_access => check!(
                parse_storage_access(expr, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::if_let_exp => check!(
                parse_if_let(expr, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            Rule::built_in_expr => check!(
                parse_built_in_expr(expr, config),
                return err(warnings, errors),
                warnings,
                errors
            ),
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(
                    a,
                    Span::from_pest(expr.as_span(), path.clone()),
                ));
                // construct unit expression for error recovery
                ParserLifter::empty(error_recovery_exp(Span::from_pest(expr.as_span(), path)))
            }
        };
        ok(parsed_result, warnings, errors)
    }
}

fn convert_unary_to_fn_calls(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let iter = item.into_inner();
    let mut unary_stack = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut expr_result = None;
    for item in iter {
        match item.as_rule() {
            Rule::unary_op => unary_stack.push((
                Span::from_pest(item.as_span(), config.map(|c| c.path())),
                check!(
                    UnaryOp::parse_from_pair(item, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
            )),
            _ => {
                expr_result = Some(check!(
                    Expression::parse_from_pair_inner(item, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ))
            }
        }
    }

    let mut expr_result = expr_result.expect("guaranteed by grammar");
    assert!(!unary_stack.is_empty(), "guaranteed by grammar");
    while let Some((op_span, unary_op)) = unary_stack.pop() {
        let exp = unary_op.to_fn_application(
            expr_result.value.clone(),
            Span::join(op_span.clone(), expr_result.value.span()),
            op_span,
        );
        expr_result = ParserLifter {
            var_decls: expr_result.var_decls,
            value: exp,
        };
    }
    ok(expr_result, warnings, errors)
}

pub(crate) fn parse_storage_access(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    debug_assert!(item.as_rule() == Rule::storage_access);
    let mut warnings = vec![];
    let mut errors = vec![];
    let path = config.map(|c| c.path());
    let span = item.as_span();
    let span = Span::from_pest(span, path);
    let mut parts = item.into_inner();
    let _storage_keyword = parts.next();

    let mut field_names = Vec::new();
    for item in parts {
        field_names.push(check!(
            ident::parse_from_pair(item, config),
            continue,
            warnings,
            errors
        ))
    }

    let exp = Expression::StorageAccess { field_names, span };
    ok(
        ParserLifter {
            var_decls: vec![],
            value: exp,
        },
        warnings,
        errors,
    )
}
pub(crate) fn parse_array_index(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let path = config.map(|c| c.path());
    let span = item.as_span();
    let mut inner_iter = item.into_inner();
    let prefix_result = check!(
        parse_call_item(inner_iter.next().unwrap(), config),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut index_result_buf = vec![];
    for index in inner_iter {
        index_result_buf.push(check!(
            Expression::parse_from_pair(index, config),
            return err(warnings, errors),
            warnings,
            errors
        ));
    }
    let mut first_result_index = index_result_buf
        .first()
        .expect("guarenteed by grammer")
        .to_owned();
    let mut var_decls = prefix_result.var_decls;
    var_decls.append(&mut first_result_index.var_decls);
    let mut exp = Expression::ArrayIndex {
        prefix: Box::new(prefix_result.value),
        index: Box::new(first_result_index.value.to_owned()),
        span: Span::from_pest(span.clone(), path.clone()),
    };
    for mut index_result in index_result_buf.into_iter().skip(1) {
        var_decls.append(&mut index_result.var_decls);
        exp = Expression::ArrayIndex {
            prefix: Box::new(exp),
            index: Box::new(index_result.value),
            span: Span::from_pest(span.clone(), path.clone()),
        };
    }
    ok(
        ParserLifter {
            var_decls,
            value: exp,
        },
        warnings,
        errors,
    )
}

pub(crate) fn parse_built_in_expr(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let span = Span::from_pest(item.as_span(), config.map(|c| c.path()));
    let mut iter = item.into_inner();
    let size_of = iter.next().expect("gaurenteed by grammar");
    let exp = match size_of.as_rule() {
        Rule::size_of_val_expr => {
            let mut inner_iter = size_of.into_inner();
            let _keyword = inner_iter.next();
            let elem = inner_iter.next().expect("guarenteed by grammar");
            let expr_result = check!(
                Expression::parse_from_pair(elem, config),
                return err(warnings, errors),
                warnings,
                errors
            );
            let exp = Expression::SizeOfVal {
                exp: Box::new(expr_result.value),
                span,
            };
            ParserLifter {
                var_decls: expr_result.var_decls,
                value: exp,
            }
        }
        // The size_of_type and is_ref_type_expr rules have identical grammar apart from the
        // keyword.
        Rule::size_of_type_expr | Rule::is_ref_type_expr => {
            let mut inner_iter = size_of.into_inner();
            let keyword = inner_iter.next().expect("guaranteed by grammar");
            let elem = inner_iter.next().expect("guaranteed by grammar");
            let type_span = Span::from_pest(elem.as_span(), config.map(|c| c.path()));
            let type_name = check!(
                TypeInfo::parse_from_pair(elem, config),
                TypeInfo::ErrorRecovery,
                warnings,
                errors
            );
            let exp = Expression::BuiltinGetTypeProperty {
                builtin: match keyword.as_str() {
                    "size_of" => BuiltinProperty::SizeOfType,
                    "is_reference_type" => BuiltinProperty::IsRefType,
                    _otherwise => unreachable!("unexpected built in keyword: {keyword}"),
                },
                type_name,
                type_span,
                span,
            };
            ParserLifter::empty(exp)
        }
        _ => unreachable!(),
    };
    ok(exp, warnings, errors)
}

fn parse_subfield_path(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let warnings = vec![];
    let mut errors = vec![];
    let path = config.map(|c| c.path());
    let item = item.into_inner().next().expect("guarenteed by grammar");
    match item.as_rule() {
        Rule::call_item => parse_call_item(item, config),
        Rule::array_index => parse_array_index(item, config),
        a => {
            eprintln!(
                "Unimplemented subfield path: {:?} ({:?}) ({:?})",
                a,
                item.as_str(),
                item.as_rule()
            );
            errors.push(CompileError::UnimplementedRule(
                a,
                Span::from_pest(item.as_span(), path.clone()),
            ));
            // construct unit expression for error recovery
            let exp_result =
                ParserLifter::empty(error_recovery_exp(Span::from_pest(item.as_span(), path)));
            ok(exp_result, warnings, errors)
        }
    }
}

// A call item is parsed as either an `ident` or a parenthesized `expr`. This method's job is to
// figure out which variant of `call_item` this is and turn it into either a variable expression
// or parse it as an expression otherwise.
fn parse_call_item(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    assert_eq!(item.as_rule(), Rule::call_item);
    let item = item.into_inner().next().expect("guaranteed by grammar");
    let exp_result = match item.as_rule() {
        Rule::ident => {
            let exp = Expression::VariableExpression {
                name: check!(
                    ident::parse_from_pair(item.clone(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                span: Span::from_pest(item.as_span(), config.map(|c| c.path())),
            };
            ParserLifter::empty(exp)
        }
        Rule::expr => check!(
            Expression::parse_from_pair(item, config),
            return err(warnings, errors),
            warnings,
            errors
        ),
        a => unreachable!("{:?}", a),
    };
    ok(exp_result, warnings, errors)
}

fn parse_array_elems(
    elems: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let path = config.map(|cfg| cfg.path());
    let span = Span::from_pest(elems.as_span(), path.clone());

    let mut elem_iter = elems.into_inner();
    let first_elem = elem_iter.next().unwrap();
    let contents = match first_elem.as_rule() {
        Rule::literal_value => {
            // The form [initialiser; count].
            let span = first_elem.as_span();
            let init = Literal::parse_from_pair(first_elem, config)
                .map(|(value, span)| ParserLifter::empty(Expression::Literal { value, span }))
                .unwrap_or_else(&mut warnings, &mut errors, || {
                    ParserLifter::empty(error_recovery_exp(Span::from_pest(span, path)))
                });

            // This is a constant integer expression we need to parse now into a count.  Currently
            // assuming it's a `u64_integer` in the grammar, so we can just use the builtin
            // `parse()` to get it.
            let count = elem_iter
                .next()
                .unwrap()
                .as_str()
                .trim()
                .parse::<usize>()
                .unwrap();
            let mut elems = Vec::with_capacity(count);
            elems.resize(count as usize, init);
            elems
        }
        _otherwise => {
            // The simple form [elem0, elem1, ..., elemN].
            let span = first_elem.as_span();
            let first_elem_expr_result = check!(
                Expression::parse_from_pair(first_elem, config),
                ParserLifter::empty(error_recovery_exp(Span::from_pest(span, path.clone()))),
                warnings,
                errors
            );
            elem_iter.fold(vec![first_elem_expr_result], |mut elems, pair| {
                let span = pair.as_span();
                elems.push(check!(
                    Expression::parse_from_pair(pair, config),
                    ParserLifter::empty(error_recovery_exp(Span::from_pest(span, path.clone()))),
                    warnings,
                    errors
                ));
                elems
            })
        }
    };

    let var_decls = contents
        .iter()
        .flat_map(|x| x.var_decls.clone())
        .collect::<Vec<_>>();
    let exps = contents.into_iter().map(|x| x.value).collect::<Vec<_>>();
    let exp = Expression::Array {
        contents: exps,
        span,
    };
    let parse_result = ParserLifter {
        var_decls,
        value: exp,
    };

    ok(parse_result, warnings, errors)
}

fn parse_op(op: Pair<Rule>, config: Option<&BuildConfig>) -> CompileResult<Op> {
    let path = config.map(|c| c.path());
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
        ">=" => GreaterThanOrEqualTo,
        "<=" => LessThanOrEqualTo,
        a => {
            errors.push(CompileError::ExpectedOp {
                op: a.to_string(),
                span: Span::from_pest(op.as_span(), path),
            });
            return err(Vec::new(), errors);
        }
    };
    ok(
        Op {
            span: Span::from_pest(op.as_span(), path),
            op_variant,
        },
        Vec::new(),
        errors,
    )
}

#[derive(Debug)]
pub(crate) struct Op {
    pub span: Span,
    pub op_variant: OpVariant,
}

impl Op {
    pub fn to_var_name(&self) -> Ident {
        Ident::new_with_override(self.op_variant.as_str(), self.span.clone())
    }
}

#[derive(Debug)]
pub enum OpVariant {
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
    GreaterThanOrEqualTo,
    LessThanOrEqualTo,
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
            Or => "$or$",
            And => "$and$",
            Equals => "eq",
            NotEquals => "neq",
            Xor => "xor",
            BinaryOr => "binary_or",
            BinaryAnd => "binary_and",
            GreaterThan => "gt",
            LessThan => "lt",
            LessThanOrEqualTo => "le",
            GreaterThanOrEqualTo => "ge",
        }
    }
    fn precedence(&self) -> usize {
        use OpVariant::*;
        // a higher number means the operation has higher precedence
        match self {
            Or => 0,
            And => 0,

            Equals => 1,
            NotEquals => 1,

            GreaterThan => 2,
            LessThan => 2,
            GreaterThanOrEqualTo => 2,
            LessThanOrEqualTo => 2,

            Add => 3,
            Subtract => 3,

            Divide => 4,
            Multiply => 4,
            Modulo => 4,

            BinaryOr => 5,
            BinaryAnd => 5,
            Xor => 5,
        }
    }
}

fn arrange_by_order_of_operations(
    expression_results: Vec<Either<Op, ParserLifter<Expression>>>,
    debug_span: Span,
) -> CompileResult<ParserLifter<Expression>> {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    let mut expression_result_stack: Vec<ParserLifter<Expression>> = Vec::new();
    let mut op_stack = Vec::new();

    for expr_result_or_op in expression_results {
        match expr_result_or_op {
            Either::Left(op) => {
                if op.op_variant.precedence()
                    < op_stack
                        .last()
                        .map(|x: &Op| x.op_variant.precedence())
                        .unwrap_or(0)
                {
                    let rhs = expression_result_stack.pop();
                    let lhs = expression_result_stack.pop();
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
                    let mut rhs = rhs.unwrap();

                    // We special case `&&` and `||` here because they are binary operators and are
                    // bound by the precedence rules, but they are not overloaded by std::ops since
                    // they must be evaluated lazily.
                    let mut new_var_decls = lhs.var_decls;
                    new_var_decls.append(&mut rhs.var_decls);
                    let new_exp = match new_op.op_variant {
                        OpVariant::And | OpVariant::Or => Expression::LazyOperator {
                            op: LazyOp::from(new_op.op_variant),
                            lhs: Box::new(lhs.value),
                            rhs: Box::new(rhs.value),
                            span: debug_span.clone(),
                        },
                        _ => Expression::core_ops(
                            new_op,
                            vec![lhs.value, rhs.value],
                            debug_span.clone(),
                        ),
                    };
                    expression_result_stack.push(ParserLifter {
                        var_decls: new_var_decls,
                        value: new_exp,
                    });
                }
                op_stack.push(op)
            }
            Either::Right(expr_result) => expression_result_stack.push(expr_result),
        }
    }

    while let Some(op) = op_stack.pop() {
        let rhs = expression_result_stack.pop();
        let lhs = expression_result_stack.pop();

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
        let mut rhs = rhs.unwrap();

        // See above about special casing `&&` and `||`.
        let span = Span::join(
            Span::join(lhs.value.span(), op.span.clone()),
            rhs.value.span(),
        );
        let mut new_var_decls = lhs.var_decls;
        new_var_decls.append(&mut rhs.var_decls);
        let new_exp = match op.op_variant {
            OpVariant::And | OpVariant::Or => Expression::LazyOperator {
                op: LazyOp::from(op.op_variant),
                lhs: Box::new(lhs.value),
                rhs: Box::new(rhs.value),
                span,
            },
            _ => Expression::core_ops(op, vec![lhs.value.clone(), rhs.value.clone()], span),
        };
        expression_result_stack.push(ParserLifter {
            var_decls: new_var_decls,
            value: new_exp,
        });
    }

    if expression_result_stack.len() != 1 {
        errors.push(CompileError::Internal(
            "Invalid expression stack length",
            debug_span,
        ));
        return err(warnings, errors);
    }

    ok(expression_result_stack[0].clone(), warnings, errors)
}

struct MatchedBranch {
    result: Expression,
    match_req_map: Vec<(Expression, Expression)>,
    match_impl_map: Vec<(Ident, Expression)>,
    branch_span: Span,
}

/// This algorithm desugars match expressions to if statements.
///
/// Given the following example:
///
/// ```ignore
/// struct Point {
///     x: u64,
///     y: u64
/// }
///
/// let p = Point {
///     x: 42,
///     y: 24
/// };
///
/// match p {
///     Point { x, y: 5 } => { x },
///     Point { x, y: 24 } => { x },
///     _ => 0
/// }
/// ```
///
/// The resulting if statement would look roughly like this:
///
/// ```ignore
/// let NEW_NAME = p;
/// if NEW_NAME.y==5 {
///     let x = 42;
///     x
/// } else if NEW_NAME.y==42 {
///     let x = 42;
///     x
/// } else {
///     0
/// }
/// ```
///
/// The steps of the algorithm can roughly be broken down into:
///
/// 0. Create a VariableDeclaration that assigns the primary expression to a variable.
/// 1. Assemble the "matched branches."
/// 2. Assemble the possibly nested giant if statement using the matched branches.
///     2a. Assemble the conditional that goes in the if primary expression.
///     2b. Assemble the statements that go inside of the body of the if expression
///     2c. Assemble the giant if statement.
/// 3. Return!
pub(crate) fn desugar_match_expression(
    primary_expression: &Expression,
    branches: Vec<MatchBranch>,
    config: Option<&BuildConfig>,
) -> CompileResult<(Expression, Ident, Vec<MatchCondition>)> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 0. Create a VariableDeclaration that assigns the primary expression to a variable.
    let var_decl_span = primary_expression.span();
    let var_decl_name = ident::random_name(var_decl_span.clone(), config);
    let var_decl_exp = Expression::VariableExpression {
        name: var_decl_name.clone(),
        span: var_decl_span,
    };

    // 1. Assemble the "matched branches."
    let mut matched_branches = vec![];
    for MatchBranch {
        condition,
        result,
        span: branch_span,
    } in branches.iter()
    {
        let matches = match condition {
            MatchCondition::CatchAll(_) => Some((vec![], vec![])),
            MatchCondition::Scrutinee(scrutinee) => check!(
                matcher(&var_decl_exp, scrutinee),
                return err(warnings, errors),
                warnings,
                errors
            ),
        };
        match matches {
            Some((match_req_map, match_impl_map)) => {
                matched_branches.push(MatchedBranch {
                    result: result.to_owned(),
                    match_req_map,
                    match_impl_map,
                    branch_span: branch_span.to_owned(),
                });
            }
            None => {
                let errors = vec![CompileError::Internal("found None", branch_span.clone())];
                let exp = Expression::Tuple {
                    fields: vec![],
                    span: branch_span.clone(),
                };
                return ok((exp, var_decl_name, vec![]), vec![], errors);
            }
        }
    }

    // 2. Assemble the possibly nested giant if statement using the matched branches.
    let mut if_statement = None;
    for MatchedBranch {
        result,
        match_req_map,
        match_impl_map,
        branch_span,
    } in matched_branches.iter().rev()
    {
        // 2a. Assemble the conditional that goes in the if primary expression.
        let mut conditional = None;
        for (left_req, right_req) in match_req_map.iter() {
            let joined_span = Span::join(left_req.clone().span(), right_req.clone().span());
            let condition = Expression::core_ops_eq(
                vec![left_req.to_owned(), right_req.to_owned()],
                joined_span,
            );
            match conditional {
                None => {
                    conditional = Some(condition);
                }
                Some(the_conditional) => {
                    conditional = Some(Expression::LazyOperator {
                        op: crate::LazyOp::And,
                        lhs: Box::new(the_conditional.clone()),
                        rhs: Box::new(condition.clone()),
                        span: Span::join(the_conditional.span(), condition.span()),
                    });
                }
            }
        }

        // 2b. Assemble the statements that go inside of the body of the if expression
        let mut code_block_stmts = vec![];
        let mut code_block_stmts_span = None;
        for (left_impl, right_impl) in match_impl_map.iter() {
            let decl = Declaration::VariableDeclaration(VariableDeclaration {
                name: left_impl.clone(),
                is_mutable: false,
                body: right_impl.clone(),
                type_ascription: TypeInfo::Unknown,
                type_ascription_span: None,
            });
            let new_span = Span::join(left_impl.span().clone(), right_impl.span());
            code_block_stmts.push(AstNode {
                content: AstNodeContent::Declaration(decl),
                span: new_span.clone(),
            });
            code_block_stmts_span = match code_block_stmts_span {
                None => Some(new_span),
                Some(old_span) => Some(Span::join(old_span, new_span)),
            };
        }
        match result {
            Expression::CodeBlock {
                contents:
                    CodeBlock {
                        contents,
                        whole_block_span,
                    },
                span: _,
            } => {
                let mut contents = contents.clone();
                code_block_stmts.append(&mut contents);
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(whole_block_span.clone()),
                    Some(old_span) => Some(Span::join(old_span, whole_block_span.clone())),
                };
            }
            result => {
                code_block_stmts.push(AstNode {
                    content: AstNodeContent::Expression(result.clone()),
                    span: result.span(),
                });
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(result.span()),
                    Some(old_span) => Some(Span::join(old_span, result.span())),
                };
            }
        }
        let code_block_stmts_span = match code_block_stmts_span {
            None => branch_span.clone(),
            Some(span) => span,
        };
        let code_block = Expression::CodeBlock {
            contents: CodeBlock {
                contents: code_block_stmts.clone(),
                whole_block_span: code_block_stmts_span.clone(),
            },
            span: code_block_stmts_span,
        };

        // 2c. Assemble the giant if statement.
        match if_statement {
            None => {
                if_statement = match conditional {
                    None => Some(code_block),
                    Some(conditional) => Some(Expression::IfExp {
                        condition: Box::new(conditional.clone()),
                        then: Box::new(code_block.clone()),
                        r#else: None,
                        span: Span::join(conditional.span(), code_block.span()),
                    }),
                };
            }
            Some(Expression::CodeBlock {
                contents: right_block,
                span: exp_span,
            }) => {
                let right = Expression::CodeBlock {
                    contents: right_block,
                    span: exp_span,
                };
                if_statement = match conditional {
                    None => Some(Expression::IfExp {
                        condition: Box::new(Expression::Literal {
                            value: Literal::Boolean(true),
                            span: branch_span.clone(),
                        }),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: Span::join(code_block.clone().span(), right.clone().span()),
                    }),
                    Some(the_conditional) => Some(Expression::IfExp {
                        condition: Box::new(the_conditional),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: Span::join(code_block.clone().span(), right.clone().span()),
                    }),
                };
            }
            Some(Expression::IfExp {
                condition,
                then,
                r#else,
                span: exp_span,
            }) => {
                if_statement = Some(Expression::IfExp {
                    condition: Box::new(conditional.unwrap()),
                    then: Box::new(code_block.clone()),
                    r#else: Some(Box::new(Expression::IfExp {
                        condition,
                        then,
                        r#else,
                        span: exp_span.clone(),
                    })),
                    span: Span::join(code_block.clone().span(), exp_span),
                });
            }
            Some(if_statement) => {
                eprintln!("Unimplemented if_statement_pattern: {:?}", if_statement,);
                errors.push(CompileError::Unimplemented(
                    "this desugared if expression pattern is not implemented",
                    if_statement.span(),
                ));
                // construct unit expression for error recovery
                let exp = Expression::Tuple {
                    fields: vec![],
                    span: if_statement.span(),
                };
                return ok((exp, var_decl_name, vec![]), warnings, errors);
            }
        }
    }

    // 3. Return!
    let cases_covered = branches
        .into_iter()
        .map(|x| x.condition)
        .collect::<Vec<_>>();
    match if_statement {
        None => err(vec![], vec![]),
        Some(if_statement) => ok(
            (if_statement, var_decl_name, cases_covered),
            warnings,
            errors,
        ),
    }
}

fn parse_if_let(
    expr: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParserLifter<Expression>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let path = config.map(|c| c.path());

    let span = Span::from_pest(expr.as_span(), path.clone());

    let mut if_exp_pairs = expr.into_inner();

    let _let_keyword = if_exp_pairs.next();
    let scrutinee_pair = if_exp_pairs.next().expect("guaranteed by grammar");
    let expr_pair = if_exp_pairs.next().expect("guaranteed by grammar");
    let then_branch_pair = if_exp_pairs.next().expect("guaranteed by grammar");
    let maybe_else_branch_pair = if_exp_pairs.next(); // not expect because this could be None and be valid

    let scrutinee = check!(
        Scrutinee::parse_from_pair(scrutinee_pair, config),
        return err(warnings, errors),
        warnings,
        errors
    );

    let ParserLifter {
        mut var_decls,
        value: expr,
    } = check!(
        Expression::parse_from_pair(expr_pair, config),
        return err(warnings, errors),
        warnings,
        errors
    );

    let then_branch_span = Span::from_pest(then_branch_pair.as_span(), path.clone());

    let then = check!(
        CodeBlock::parse_from_pair(then_branch_pair, config),
        crate::CodeBlock {
            contents: Vec::new(),
            whole_block_span: then_branch_span,
        },
        warnings,
        errors
    );

    let maybe_else_branch = if let Some(ref else_branch) = maybe_else_branch_pair {
        let else_span = Span::from_pest(else_branch.as_span(), path);
        match else_branch.as_rule() {
            Rule::code_block => {
                let block = check!(
                    CodeBlock::parse_from_pair(else_branch.clone(), config),
                    CodeBlock {
                        contents: Vec::new(),
                        whole_block_span: else_span.clone(),
                    },
                    warnings,
                    errors
                );
                let exp = Expression::CodeBlock {
                    contents: block,
                    span: else_span,
                };
                Some(Box::new(exp))
            }
            Rule::if_let_exp => {
                let ParserLifter {
                    var_decls: mut var_decls2,
                    value: r#else,
                } = check!(
                    parse_if_let(else_branch.clone(), config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                var_decls.append(&mut var_decls2);
                Some(Box::new(r#else))
            }
            _ => unreachable!("guaranteed by grammar"),
        }
    } else {
        None
    };
    let exp = Expression::IfLet {
        scrutinee,
        expr: Box::new(expr),
        then,
        r#else: maybe_else_branch,
        span,
    };
    let result = ParserLifter {
        var_decls,
        value: exp,
    };
    ok(result, warnings, errors)
}
