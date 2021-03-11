use super::*;
use crate::error::*;
use crate::parse_tree::*;
use crate::types::{IntegerBits, TypeInfo};
use crate::{AstNode, AstNodeContent, CodeBlock, ParseTree, ReturnStatement, TraitFn};
use either::Either;
use pest::Span;
use std::collections::HashMap;

pub(crate) const ERROR_RECOVERY_EXPR: TypedExpression = TypedExpression {
    expression: TypedExpressionVariant::Unit,
    return_type: TypeInfo::ErrorRecovery,
    is_constant: IsConstant::No,
};

#[derive(Clone, Debug)]
pub(crate) struct TypedExpression<'sc> {
    pub(crate) expression: TypedExpressionVariant<'sc>,
    pub(crate) return_type: TypeInfo<'sc>,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
}
#[derive(Clone, Debug)]
pub(crate) enum TypedExpressionVariant<'sc> {
    Literal(Literal<'sc>),
    FunctionApplication {
        name: VarName<'sc>,
        arguments: Vec<TypedExpression<'sc>>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
        name: VarName<'sc>,
    },
    Unit,
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    MatchExpression {
        primary_expression: Box<TypedExpression<'sc>>,
        branches: Vec<TypedMatchBranch<'sc>>,
    },
    StructExpression {
        struct_name: &'sc str,
        fields: Vec<TypedStructExpressionField<'sc>>,
    },
    CodeBlock(TypedCodeBlock<'sc>),
    // a flag that this value will later be provided as a parameter, but is currently unknown
    FunctionParameter,
    IfExp {
        condition: Box<TypedExpression<'sc>>,
        then: Box<TypedExpression<'sc>>,
        r#else: Option<Box<TypedExpression<'sc>>>,
    },
    AsmExpression {
        asm: AsmExpression<'sc>,
    },
}
#[derive(Clone, Debug)]
pub(crate) struct TypedStructExpressionField<'sc> {
    name: &'sc str,
    value: TypedExpression<'sc>,
}
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    condition: TypedMatchCondition<'sc>,
    result: Either<TypedCodeBlock<'sc>, TypedExpression<'sc>>,
}

#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}

impl<'sc> TypedExpression<'sc> {
    pub(crate) fn type_check(
        other: Expression<'sc>,
        namespace: &HashMap<VarName<'sc>, TypedDeclaration<'sc>>,
        methods_namespace: &HashMap<TypeInfo<'sc>, Vec<TypedFunctionDeclaration<'sc>>>,
        type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_span = other.span();
        let typed_expression = match other {
            Expression::Literal { value: lit, .. } => {
                let return_type = match lit {
                    Literal::String(_) => TypeInfo::String,
                    Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                    Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                    Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                    Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                    Literal::U128(_) => TypeInfo::UnsignedInteger(IntegerBits::OneTwentyEight),
                    Literal::Boolean(_) => TypeInfo::Boolean,
                    Literal::Byte(_) => TypeInfo::Byte,
                    Literal::Byte32(_) => TypeInfo::Byte32,
                };
                TypedExpression {
                    expression: TypedExpressionVariant::Literal(lit),
                    return_type,
                    is_constant: IsConstant::Yes,
                }
            }
            Expression::VariableExpression { name, unary_op, .. } => match namespace.get(&name) {
                Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    body,
                    ..
                })) => TypedExpression {
                    return_type: body.return_type.clone(),
                    is_constant: body.is_constant,
                    expression: TypedExpressionVariant::VariableExpression {
                        unary_op: unary_op.clone(),
                        name: name.clone(),
                    },
                },
                Some(a) => {
                    errors.push(CompileError::NotAVariable {
                        name: name.span.as_str(),
                        span: name.span,
                        what_it_is: a.friendly_name(),
                    });
                    ERROR_RECOVERY_EXPR.clone()
                }
                None => {
                    errors.push(CompileError::UnknownVariable {
                        var_name: name.span.as_str().trim(),
                        span: name.span,
                    });
                    ERROR_RECOVERY_EXPR.clone()
                }
            },
            Expression::FunctionApplication {
                name, arguments, ..
            } => {
                let function_declaration = namespace.get(&name);
                match function_declaration {
                    Some(TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                        parameters,
                        return_type,
                        ..
                    })) => {
                        // type check arguments in function application vs arguments in function
                        // declaration. Use parameter type annotations as annotations for the
                        // arguments
                        //
                        let mut typed_call_arguments = Vec::new();
                        for (arg, param) in arguments.into_iter().zip(parameters.iter()) {
                            let res = TypedExpression::type_check(
                                arg,
                                &namespace,
                                methods_namespace,
                                Some(param.r#type.clone()),
                                    "The argument that has been provided to this function's type does not match the declared type of the parameter in the function declaration."
                            );
                            let arg = match res {
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
                                    ERROR_RECOVERY_EXPR.clone()
                                }
                            };
                            typed_call_arguments.push(arg);
                        }

                        TypedExpression {
                            return_type: return_type.clone(),
                            // now check the function call return type
                            // FEATURE this IsConstant can be true if the function itself is constant-able
                            // const functions would be an advanced feature and are not supported right
                            // now
                            is_constant: IsConstant::No,
                            expression: TypedExpressionVariant::FunctionApplication {
                                arguments: typed_call_arguments,
                                name: name.clone(),
                            },
                        }
                    }
                    Some(a) => {
                        errors.push(CompileError::NotAFunction {
                            name: name.span.as_str(),
                            span: name.span,
                            what_it_is: a.friendly_name(),
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                    None => {
                        errors.push(CompileError::UnknownFunction {
                            name: name.span.as_str(),
                            span: name.span,
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                }
            }
            Expression::MatchExpression {
                primary_expression,
                branches,
                span,
                ..
            } => {
                let typed_primary_expression = type_check!(
                    TypedExpression,
                    *primary_expression,
                    &namespace,
                    &methods_namespace,
                    None,
                    "",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                let first_branch_result = type_check!(
                    TypedExpression,
                    branches[0].result.clone(),
                    &namespace,
                    &methods_namespace,
                    type_annotation.clone(),
                    help_text.clone(),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );

                let first_branch_result = vec![first_branch_result];
                // use type of first branch for annotation on the rest of the branches
                // we checked the first branch separately just to get its return type for inferencing the rest
                let mut rest_of_branches = branches
                    .into_iter()
                    .skip(1)
                    .map(
                        |MatchBranch {
                             condition, result, ..
                         }| {
                            type_check!(
                                TypedExpression,
                                result,
                                &namespace,
                                &methods_namespace,
                                Some(first_branch_result[0].return_type.clone()),
                                "All branches of a match expression must be of the same type.",
                                ERROR_RECOVERY_EXPR.clone(),
                                warnings,
                                errors
                            )
                        },
                    )
                    .collect::<Vec<_>>();

                let mut all_branches = first_branch_result;
                all_branches.append(&mut rest_of_branches);

                errors.push(CompileError::Unimplemented(
                    "Match expressions and pattern matching",
                    span,
                ));
                ERROR_RECOVERY_EXPR.clone()
            }
            Expression::CodeBlock { contents, .. } => {
                let (typed_block, block_return_type) = type_check!(
                    TypedCodeBlock,
                    contents.clone(),
                    &namespace,
                    &methods_namespace,
                    type_annotation.clone(),
                    help_text.clone(),
                    (TypedCodeBlock { contents: vec![] }, TypeInfo::Unit),
                    warnings,
                    errors
                );
                TypedExpression {
                    expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                        contents: typed_block.contents,
                    }),
                    return_type: block_return_type,
                    is_constant: IsConstant::No, // TODO if all elements of block are constant then this is constant
                }
            }
            // TODO if _condition_ is constant, evaluate it and compile this to a regular
            // expression with only one branch
            Expression::IfExp {
                condition,
                then,
                r#else,
                span,
            } => {
                let condition = Box::new(type_check!(
                    TypedExpression,
                    *condition,
                    &namespace,
                    &methods_namespace,
                    Some(TypeInfo::Boolean),
                    "The condition of an if expression must be a boolean expression.",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let then = Box::new(type_check!(
                    TypedExpression,
                    *then,
                    &namespace,
                    &methods_namespace,
                    None,
                    "",
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let r#else = if let Some(expr) = r#else {
                    Some(Box::new(type_check!(
                        TypedExpression,
                        *expr,
                        namespace,
                        &methods_namespace,
                        Some(then.return_type.clone()),
                        "",
                        ERROR_RECOVERY_EXPR.clone(),
                        warnings,
                        errors
                    )))
                } else {
                    None
                };

                TypedExpression {
                    expression: TypedExpressionVariant::IfExp {
                        condition,
                        then: then.clone(),
                        r#else,
                    },
                    is_constant: IsConstant::No, // TODO
                    return_type: then.return_type,
                }
            }
            Expression::AsmExpression { span, asm } => {
                let return_type = if asm.returns.is_some() {
                    TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)
                } else {
                    TypeInfo::Unit
                };
                TypedExpression {
                    expression: TypedExpressionVariant::AsmExpression { asm },
                    return_type,
                    is_constant: IsConstant::No,
                }
            }
            a => {
                println!("Unimplemented: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                ));

                ERROR_RECOVERY_EXPR
            }
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            let convertability = typed_expression.return_type.clone().is_convertable(
                type_annotation.clone(),
                expr_span.clone(),
                help_text,
            );
            match convertability {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: expr_span,
                        });
                    }
                }
                Err(err) => {
                    errors.push(err.into());
                }
            }
        }
        ok(typed_expression, warnings, errors)
    }
}
