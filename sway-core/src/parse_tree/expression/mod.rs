use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{ident, CallPath, Literal},
    parser::Rule,
    type_engine::TypeInfo,
    AstNode, AstNodeContent, CodeBlock, Declaration, VariableDeclaration,
};

use sway_types::{ident::Ident, join_spans, Span};

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
pub(crate) use method_name::MethodName;
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
        type_arguments: Vec<(TypeInfo, Span)>,
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
    Array {
        contents: Vec<Expression>,
        span: Span,
    },
    StructExpression {
        struct_name: Ident,
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
        span: Span,
    },
    // separated into other struct for parsing reasons
    AsmExpression {
        span: Span,
        asm: AsmExpression,
    },
    MethodApplication {
        method_name: MethodName,
        arguments: Vec<Expression>,
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
        type_arguments: Vec<TypeInfo>,
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

/// During type checking, this gets replaced with enum arg access.
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

#[derive(Clone, Debug, PartialEq)]
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
    pub(crate) name: Ident,
    pub(crate) value: Expression,
    pub(crate) span: Span,
}

pub(crate) fn error_recovery_parse_result<T>(value: T) -> ParseResult<T> {
    ParseResult {
        var_decls: vec![],
        value,
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
                },
                type_name: None,
                is_absolute: true,
            },
            arguments,
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
                },
                type_name: None,
                is_absolute: true,
            },
            arguments,
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
        })
        .clone()
    }

    pub(crate) fn parse_from_pair(
        expr: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParseResult<Self>> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_for_debug = expr.clone();
        let mut expr_iter = expr.into_inner();
        // first expr is always here
        let first_expr = expr_iter.next().unwrap();
        let first_expr_result = check!(
            Expression::parse_from_pair_inner(first_expr.clone(), config),
            error_recovery_parse_result(Expression::Tuple {
                fields: vec![],
                span: Span {
                    span: first_expr.as_span(),
                    path: path.clone(),
                }
            }),
            warnings,
            errors
        );
        let mut expr_result_or_op_buf: Vec<Either<Op, ParseResult<Expression>>> =
            vec![Either::Right(first_expr_result.clone())];
        // sometimes exprs are followed by ops in the same expr
        while let Some(op) = expr_iter.next() {
            let op_str = op.as_str().to_string();
            let op_span = Span {
                span: op.as_span(),
                path: path.clone(),
            };

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
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: Span {
                            span: o.as_span(),
                            path: path.clone()
                        }
                    }),
                    warnings,
                    errors
                ),
                None => {
                    errors.push(CompileError::ExpectedExprAfterOp {
                        op: op_str,
                        span: Span {
                            span: expr_for_debug.as_span(),
                            path: path.clone(),
                        },
                    });
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: op_span,
                    })
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
                Span {
                    span: expr_for_debug.as_span(),
                    path: path.clone(),
                },
            )
            .unwrap_or_else(&mut warnings, &mut errors, || {
                error_recovery_parse_result(Expression::Tuple {
                    fields: vec![],
                    span: Span {
                        span: expr_for_debug.as_span(),
                        path: path.clone(),
                    },
                })
            });
            ok(expr_result, warnings, errors)
        }
    }

    pub(crate) fn parse_from_pair_inner(
        expr: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<ParseResult<Self>> {
        let path = config.map(|c| c.path());
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let span = Span {
            span: expr.as_span(),
            path: path.clone(),
        };
        #[allow(unused_assignments)]
        let mut maybe_type_args = Vec::new();
        let parsed_result = match expr.as_rule() {
            Rule::literal_value => Literal::parse_from_pair(expr, config)
                .map(|(value, span)| ParseResult {
                    var_decls: vec![],
                    value: Expression::Literal { value, span },
                })
                .unwrap_or_else(&mut warnings, &mut errors, || {
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span,
                    })
                }),
            Rule::func_app => {
                let span = Span {
                    span: expr.as_span(),
                    path: path.clone(),
                };
                let mut func_app_parts = expr.into_inner();
                let first_part = func_app_parts.next().unwrap();
                assert!(first_part.as_rule() == Rule::ident);
                let suffix = check!(
                    ident::parse_from_pair(first_part, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let name = CallPath {
                    prefixes: vec![],
                    suffix,
                };
                let (arguments, type_args) = {
                    let maybe_type_args = func_app_parts.next().unwrap();
                    match maybe_type_args.as_rule() {
                        Rule::type_args => (func_app_parts.next().unwrap(), Some(maybe_type_args)),
                        Rule::fn_args => (maybe_type_args, None),
                        _ => unreachable!(),
                    }
                };
                maybe_type_args = type_args
                    .map(|x| x.into_inner().skip(1).collect::<Vec<_>>())
                    .unwrap_or_else(Vec::new);
                let mut arguments_buf = Vec::new();
                for argument in arguments.into_inner() {
                    let arg = check!(
                        Expression::parse_from_pair(argument.clone(), config),
                        error_recovery_parse_result(Expression::Tuple {
                            fields: vec![],
                            span: Span {
                                span: argument.as_span(),
                                path: path.clone()
                            }
                        }),
                        warnings,
                        errors
                    );
                    arguments_buf.push(arg);
                }
                let mut type_args_buf = vec![];
                for arg in maybe_type_args {
                    let sp = Span {
                        span: arg.as_span(),
                        path: path.clone(),
                    };
                    type_args_buf.push((
                        check!(
                            TypeInfo::parse_from_pair(arg.into_inner().next().unwrap(), config),
                            return err(warnings, errors),
                            warnings,
                            errors
                        ),
                        sp,
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
                    type_arguments: type_args_buf,
                };
                ParseResult {
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
                ParseResult {
                    var_decls: vec![],
                    value: exp,
                }
            }
            Rule::array_exp => match expr.into_inner().next() {
                None => ParseResult {
                    var_decls: vec![],
                    value: Expression::Array {
                        contents: Vec::new(),
                        span,
                    },
                },
                Some(array_elems) => check!(
                    parse_array_elems(array_elems, config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span,
                    }),
                    warnings,
                    errors
                ),
            },
            Rule::match_expression => {
                let mut expr_iter = expr.into_inner();
                let primary_expression_result = check!(
                    Expression::parse_from_pair(expr_iter.next().unwrap(), config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: span.clone()
                    }),
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
                let mut if_exp_result = check!(
                    desugar_match_expression(
                        primary_expression_result.value,
                        branches,
                        span.clone(),
                        config
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut var_decls = primary_expression_result.var_decls;
                var_decls.append(&mut if_exp_result.var_decls);
                let exp = Expression::MatchExp {
                    if_exp: Box::new(if_exp_result.value),
                    span,
                };
                ParseResult {
                    var_decls,
                    value: exp,
                }
            }
            Rule::struct_expression => {
                let mut expr_iter = expr.into_inner();
                let struct_name = expr_iter.next().unwrap();
                let struct_name = check!(
                    ident::parse_from_pair(struct_name, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
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
                    let span = Span {
                        span: fields[i].as_span(),
                        path: path.clone(),
                    };
                    let mut value_result = check!(
                        Expression::parse_from_pair(fields[i + 1].clone(), config),
                        error_recovery_parse_result(Expression::Tuple {
                            fields: vec![],
                            span: span.clone()
                        }),
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
                    fields: fields_buf,
                    span,
                };
                ParseResult {
                    var_decls,
                    value: exp,
                }
            }
            Rule::parenthesized_expression => {
                check!(
                    Expression::parse_from_pair(expr.clone().into_inner().next().unwrap(), config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: Span {
                            span: expr.as_span(),
                            path,
                        }
                    }),
                    warnings,
                    errors
                )
            }
            Rule::code_block => {
                let whole_block_span = Span {
                    span: expr.as_span(),
                    path,
                };
                let expr = check!(
                    crate::CodeBlock::parse_from_pair(expr, config),
                    crate::CodeBlock {
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
                ParseResult {
                    var_decls: vec![],
                    value: exp,
                }
            }
            Rule::if_exp => {
                let span = Span {
                    span: expr.as_span(),
                    path,
                };
                let mut if_exp_pairs = expr.into_inner();
                let condition_pair = if_exp_pairs.next().unwrap();
                let then_pair = if_exp_pairs.next().unwrap();
                let else_pair = if_exp_pairs.next();
                let condition_result = check!(
                    Expression::parse_from_pair(condition_pair, config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: span.clone()
                    }),
                    warnings,
                    errors
                );
                let mut then_result = check!(
                    Expression::parse_from_pair_inner(then_pair, config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: span.clone()
                    }),
                    warnings,
                    errors
                );
                let r#else_result = else_pair.map(|else_pair| {
                    check!(
                        Expression::parse_from_pair_inner(else_pair, config),
                        error_recovery_parse_result(Expression::Tuple {
                            fields: vec![],
                            span: span.clone()
                        }),
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
                ParseResult {
                    var_decls,
                    value: exp,
                }
            }
            Rule::asm_expression => {
                let whole_block_span = Span {
                    span: expr.as_span(),
                    path,
                };
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
                ParseResult {
                    var_decls: asm_result.var_decls,
                    value: exp,
                }
            }
            Rule::method_exp => {
                let whole_exp_span = Span {
                    span: expr.as_span(),
                    path: path.clone(),
                };
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
                        let mut argument_results_buf = VecDeque::new();
                        for argument in function_arguments {
                            let arg_result = check!(
                                Expression::parse_from_pair(argument.clone(), config),
                                error_recovery_parse_result(Expression::Tuple {
                                    fields: vec![],
                                    span: Span {
                                        span: argument.as_span(),
                                        path: path.clone()
                                    }
                                }),
                                warnings,
                                errors
                            );
                            argument_results_buf.push_back(arg_result);
                        }
                        // the first thing is either an exp or a var, everything subsequent must be
                        // a field
                        let mut name_parts = name_parts.into_iter();
                        let mut expr_result = check!(
                            parse_subfield_path(
                                name_parts.next().expect("guaranteed by grammar"),
                                config
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );

                        for name_part in name_parts {
                            let new_expr = Expression::SubfieldExpression {
                                prefix: Box::new(expr_result.value.clone()),
                                span: Span {
                                    span: name_part.as_span(),
                                    path: path.clone(),
                                },
                                field_to_access: check!(
                                    ident::parse_from_pair(name_part, config),
                                    continue,
                                    warnings,
                                    errors
                                ),
                            };
                            expr_result = ParseResult {
                                var_decls: expr_result.var_decls,
                                value: new_expr,
                            };
                        }

                        argument_results_buf.push_front(expr_result);
                        let var_decls = argument_results_buf
                            .iter()
                            .flat_map(|x| x.var_decls.clone())
                            .collect::<Vec<_>>();
                        let arguments_buf = argument_results_buf
                            .into_iter()
                            .map(|x| x.value)
                            .collect::<Vec<_>>();
                        let exp = Expression::MethodApplication {
                            method_name: MethodName::FromModule { method_name },
                            arguments: arguments_buf,
                            span: whole_exp_span,
                        };
                        ParseResult {
                            var_decls,
                            value: exp,
                        }
                    }
                    Rule::fully_qualified_method => {
                        let mut path_parts_buf = vec![];
                        let mut type_name = None;
                        let mut method_name = None;
                        let mut arguments = None;
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::path_separator => (),
                                Rule::path_ident => {
                                    path_parts_buf.push(check!(
                                        ident::parse_from_pair(pair, config),
                                        continue,
                                        warnings,
                                        errors
                                    ));
                                }
                                Rule::type_name => {
                                    type_name = Some(pair);
                                }
                                Rule::call_item => {
                                    method_name = Some(pair);
                                }
                                Rule::fn_args => {
                                    arguments = Some(pair);
                                }
                                a => unreachable!("guaranteed by grammar: {:?}", a),
                            }
                        }
                        let type_name = check!(
                            TypeInfo::parse_from_pair(
                                type_name.expect("guaranteed by grammar"),
                                config
                            ),
                            TypeInfo::ErrorRecovery,
                            warnings,
                            errors
                        );

                        // parse the method name into a call path
                        let method_name = MethodName::FromType {
                            call_path: CallPath {
                                prefixes: path_parts_buf,
                                suffix: check!(
                                    ident::parse_from_pair(
                                        method_name.expect("guaranteed by grammar"),
                                        config
                                    ),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                ),
                            },
                            type_name: Some(type_name),
                            is_absolute: false,
                        };

                        let mut argument_results_buf = vec![];
                        // evaluate  the arguments passed in to the method
                        if let Some(arguments) = arguments {
                            for argument in arguments.into_inner() {
                                let arg_result = check!(
                                    Expression::parse_from_pair(argument.clone(), config),
                                    error_recovery_parse_result(Expression::Tuple {
                                        fields: vec![],
                                        span: Span {
                                            span: argument.as_span(),
                                            path: path.clone()
                                        }
                                    }),
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
                            arguments: arguments_buf,
                            span: whole_exp_span,
                        };
                        ParseResult {
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
                let span = Span {
                    span: expr.as_span(),
                    path,
                };
                let mut parts = expr.into_inner();
                let path_component = parts.next().unwrap();
                let instantiator = parts.next();
                let path = check!(
                    CallPath::parse_from_pair(path_component, config),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let arg_results = if let Some(inst) = instantiator {
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

                // if there is an expression in parenthesis, that is the instantiator.

                let var_decls = arg_results
                    .iter()
                    .flat_map(|x| x.var_decls.clone())
                    .collect::<Vec<_>>();
                let args = arg_results.into_iter().map(|x| x.value).collect::<Vec<_>>();
                let exp = Expression::DelineatedPath {
                    call_path: path,
                    args,
                    span,
                    // Eventually, when we support generic enums, we want to be able to parse type
                    // arguments on the enum name and throw them in here. TODO
                    type_arguments: vec![],
                };
                ParseResult {
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
                        error_recovery_parse_result(Expression::Tuple {
                            fields: vec![],
                            span: span.clone()
                        }),
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
                ParseResult {
                    var_decls,
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
                        span: Span {
                            span: name_part.as_span(),
                            path: path.clone(),
                        },
                        field_to_access: check!(
                            ident::parse_from_pair(name_part, config),
                            continue,
                            warnings,
                            errors
                        ),
                    };
                    expr_result = ParseResult {
                        var_decls: expr_result.var_decls,
                        value: new_expr,
                    };
                }

                expr_result
            }
            Rule::abi_cast => {
                let span = Span {
                    span: expr.as_span(),
                    path,
                };
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
                ParseResult {
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
            a => {
                eprintln!(
                    "Unimplemented expr: {:?} ({:?}) ({:?})",
                    a,
                    expr.as_str(),
                    expr.as_rule()
                );
                errors.push(CompileError::UnimplementedRule(
                    a,
                    Span {
                        span: expr.as_span(),
                        path: path.clone(),
                    },
                ));
                // construct unit expression for error recovery
                error_recovery_parse_result(Expression::Tuple {
                    fields: vec![],
                    span: Span {
                        span: expr.as_span(),
                        path,
                    },
                })
            }
        };
        ok(parsed_result, warnings, errors)
    }
}

fn convert_unary_to_fn_calls(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParseResult<Expression>> {
    let iter = item.into_inner();
    let mut unary_stack = vec![];
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut expr_result = None;
    for item in iter {
        match item.as_rule() {
            Rule::unary_op => unary_stack.push((
                Span {
                    span: item.as_span(),
                    path: config.map(|c| c.path()),
                },
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
            join_spans(op_span.clone(), expr_result.value.span()),
            op_span,
        );
        expr_result = ParseResult {
            var_decls: expr_result.var_decls,
            value: exp,
        };
    }
    ok(expr_result, warnings, errors)
}

pub(crate) fn parse_array_index(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParseResult<Expression>> {
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
        span: Span {
            span: span.clone(),
            path: path.clone(),
        },
    };
    for mut index_result in index_result_buf.into_iter().skip(1) {
        var_decls.append(&mut index_result.var_decls);
        exp = Expression::ArrayIndex {
            prefix: Box::new(exp),
            index: Box::new(index_result.value),
            span: Span {
                span: span.clone(),
                path: path.clone(),
            },
        };
    }
    ok(
        ParseResult {
            var_decls,
            value: exp,
        },
        warnings,
        errors,
    )
}

fn parse_subfield_path(
    item: Pair<Rule>,
    config: Option<&BuildConfig>,
) -> CompileResult<ParseResult<Expression>> {
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
                Span {
                    span: item.as_span(),
                    path: path.clone(),
                },
            ));
            // construct unit expression for error recovery
            let exp_result = error_recovery_parse_result(Expression::Tuple {
                fields: vec![],
                span: Span {
                    span: item.as_span(),
                    path,
                },
            });
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
) -> CompileResult<ParseResult<Expression>> {
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
                span: Span {
                    span: item.as_span(),
                    path: config.map(|c| c.path()),
                },
            };
            ParseResult {
                var_decls: vec![],
                value: exp,
            }
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
) -> CompileResult<ParseResult<Expression>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let path = config.map(|cfg| cfg.path());
    let span = Span {
        span: elems.as_span(),
        path: path.clone(),
    };

    let mut elem_iter = elems.into_inner();
    let first_elem = elem_iter.next().unwrap();
    let contents = match first_elem.as_rule() {
        Rule::literal_value => {
            // The form [initialiser; count].
            let span = first_elem.as_span();
            let init = Literal::parse_from_pair(first_elem, config)
                .map(|(value, span)| ParseResult {
                    var_decls: vec![],
                    value: Expression::Literal { value, span },
                })
                .unwrap_or_else(&mut warnings, &mut errors, || {
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: Span { span, path },
                    })
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
                error_recovery_parse_result(Expression::Tuple {
                    fields: vec![],
                    span: Span {
                        span,
                        path: path.clone()
                    }
                }),
                warnings,
                errors
            );
            elem_iter.fold(vec![first_elem_expr_result], |mut elems, pair| {
                let span = pair.as_span();
                elems.push(check!(
                    Expression::parse_from_pair(pair, config),
                    error_recovery_parse_result(Expression::Tuple {
                        fields: vec![],
                        span: Span {
                            span,
                            path: path.clone()
                        }
                    }),
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
    let parse_result = ParseResult {
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
                span: Span {
                    span: op.as_span(),
                    path,
                },
            });
            return err(Vec::new(), errors);
        }
    };
    ok(
        Op {
            span: Span {
                span: op.as_span(),
                path,
            },
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
    expression_results: Vec<Either<Op, ParseResult<Expression>>>,
    debug_span: Span,
) -> CompileResult<ParseResult<Expression>> {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    let mut expression_result_stack: Vec<ParseResult<Expression>> = Vec::new();
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
                    expression_result_stack.push(ParseResult {
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
        let span = join_spans(
            join_spans(lhs.value.span(), op.span.clone()),
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
        expression_result_stack.push(ParseResult {
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
/// if y==5 {
///     let x = 42;
///     x
/// } else if y==42 {
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
pub fn desugar_match_expression(
    primary_expression: Expression,
    branches: Vec<MatchBranch>,
    _span: Span,
    config: Option<&BuildConfig>,
) -> CompileResult<ParseResult<Expression>> {
    let mut errors = vec![];
    let mut warnings = vec![];

    // 0. Create a VariableDeclaration that assigns the primary expression to a variable.
    let var_decl_span = primary_expression.span();
    let var_decl_name = ident::random_name(var_decl_span.clone(), config);
    let var_decl = VariableDeclaration {
        name: var_decl_name.clone(),
        type_ascription: TypeInfo::Unknown,
        type_ascription_span: None,
        is_mutable: false,
        body: primary_expression,
    };
    let var_decl_exp = Expression::VariableExpression {
        name: var_decl_name,
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
                let errors = vec![CompileError::PatternMatchingAlgorithmFailure(
                    "found None",
                    branch_span.clone(),
                )];
                let exp = Expression::Tuple {
                    fields: vec![],
                    span: branch_span.clone(),
                };
                return ok(
                    ParseResult {
                        var_decls: vec![],
                        value: exp,
                    },
                    vec![],
                    errors,
                );
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
            let joined_span = join_spans(left_req.clone().span(), right_req.clone().span());
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
                        span: join_spans(the_conditional.span(), condition.span()),
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
            let new_span = join_spans(left_impl.span().clone(), right_impl.span());
            code_block_stmts.push(AstNode {
                content: AstNodeContent::Declaration(decl),
                span: new_span.clone(),
            });
            code_block_stmts_span = match code_block_stmts_span {
                None => Some(new_span),
                Some(old_span) => Some(join_spans(old_span, new_span)),
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
                    Some(old_span) => Some(join_spans(old_span, whole_block_span.clone())),
                };
            }
            result => {
                code_block_stmts.push(AstNode {
                    content: AstNodeContent::Expression(result.clone()),
                    span: result.span(),
                });
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(result.span()),
                    Some(old_span) => Some(join_spans(old_span, result.span())),
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
                        span: join_spans(conditional.span(), code_block.span()),
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
                        span: join_spans(code_block.clone().span(), right.clone().span()),
                    }),
                    Some(the_conditional) => Some(Expression::IfExp {
                        condition: Box::new(the_conditional),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: join_spans(code_block.clone().span(), right.clone().span()),
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
                    span: join_spans(code_block.clone().span(), exp_span),
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
                return ok(
                    ParseResult {
                        var_decls: vec![],
                        value: exp,
                    },
                    warnings,
                    errors,
                );
            }
        }
    }

    // 3. Return!
    match if_statement {
        None => err(vec![], vec![]),
        Some(if_statement) => ok(
            ParseResult {
                var_decls: vec![var_decl],
                value: if_statement,
            },
            warnings,
            errors,
        ),
    }
}
