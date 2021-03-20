use super::*;
use crate::types::{IntegerBits, TypeInfo};
use either::Either;
use std::collections::VecDeque;

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
        name: CallPath<'sc>,
        arguments: Vec<TypedExpression<'sc>>,
    },
    VariableExpression {
        unary_op: Option<UnaryOp>,
        name: Ident<'sc>,
    },
    Unit,
    #[allow(dead_code)]
    Array {
        contents: Vec<TypedExpression<'sc>>,
    },
    #[allow(dead_code)]
    MatchExpression {
        primary_expression: Box<TypedExpression<'sc>>,
        branches: Vec<TypedMatchBranch<'sc>>,
    },
    StructExpression {
        struct_name: Ident<'sc>,
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
    pub(crate) name: &'sc str,
    pub(crate) value: TypedExpression<'sc>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct TypedMatchBranch<'sc> {
    condition: TypedMatchCondition<'sc>,
    result: Either<TypedCodeBlock<'sc>, TypedExpression<'sc>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum TypedMatchCondition<'sc> {
    CatchAll,
    Expression(TypedExpression<'sc>),
}

impl<'sc> TypedExpression<'sc> {
    pub(crate) fn type_check<'manifest>(
        other: Expression<'sc>,
        namespace: &Namespace<'sc>,
        type_annotation: Option<TypeInfo<'sc>>,
        help_text: impl Into<String> + Clone,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_span = other.span();
        let mut typed_expression = match other {
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
            Expression::VariableExpression { name, unary_op, .. } => {
                match namespace.get_symbol(&name) {
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
                }
            }
            Expression::FunctionApplication {
                name, arguments, ..
            } => {
                todo!("fn app w/ new namespace");
                let function_declaration = namespace.get_call_path(&name);
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
                            name: name.span().as_str(),
                            span: name.span(),
                            what_it_is: a.friendly_name(),
                        });
                        ERROR_RECOVERY_EXPR.clone()
                    }
                    None => {
                        errors.push(CompileError::UnknownFunction {
                            name: name.span().as_str(),
                            span: name.span(),
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
                    TypedExpression::type_check(*primary_expression, &namespace, None, ""),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                let first_branch_result = type_check!(
                    TypedExpression::type_check(
                        branches[0].result.clone(),
                        &namespace,
                        type_annotation.clone(),
                        help_text.clone()
                    ),
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
                                TypedExpression::type_check(
                                    result,
                                    &namespace,
                                    Some(first_branch_result[0].return_type.clone()),
                                    "All branches of a match expression must be of the same type.",
                                ),
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
                    TypedCodeBlock::type_check(
                        contents.clone(),
                        &namespace,
                        type_annotation.clone(),
                        help_text.clone()
                    ),
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
                    TypedExpression::type_check(
                        *condition,
                        &namespace,
                        Some(TypeInfo::Boolean),
                        "The condition of an if expression must be a boolean expression.",
                    ),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let then = Box::new(type_check!(
                    TypedExpression::type_check(*then, &namespace, None, ""),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                ));
                let r#else = if let Some(expr) = r#else {
                    Some(Box::new(type_check!(
                        TypedExpression::type_check(
                            *expr,
                            namespace,
                            Some(then.return_type.clone()),
                            ""
                        ),
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
            Expression::StructExpression {
                span,
                struct_name,
                fields,
            } => {
                // TODO in here replace generic types with provided types
                // find the struct definition in the namespace
                let definition: &StructDeclaration = match namespace.get_symbol(&struct_name) {
                    Some(TypedDeclaration::StructDeclaration(st)) => st,
                    Some(_) => {
                        errors.push(CompileError::DeclaredNonStructAsStruct {
                            name: struct_name.primary_name,
                            span: span.clone(),
                        });
                        return err(warnings, errors);
                    }
                    None => {
                        errors.push(CompileError::StructNotFound {
                            name: struct_name.primary_name,
                            span: span.clone(),
                        });
                        return err(warnings, errors);
                    }
                };
                let mut typed_fields_buf = vec![];

                // match up the names with their type annotations from the declaration
                for def_field in definition.fields.iter() {
                    let expr_field = match fields.iter().find(|x| x.name == def_field.name) {
                        Some(val) => val.clone(),
                        None => {
                            errors.push(CompileError::StructMissingField {
                                field_name: def_field.name,
                                struct_name: definition.name.primary_name,
                                span: span.clone(),
                            });
                            typed_fields_buf.push(TypedStructExpressionField {
                                name: def_field.name,
                                value: TypedExpression {
                                    expression: TypedExpressionVariant::Unit,
                                    return_type: TypeInfo::ErrorRecovery,
                                    is_constant: IsConstant::No,
                                },
                            });
                            continue;
                        }
                    };

                    let typed_field = type_check!(
                        TypedExpression::type_check(
                        expr_field.value,
                        &namespace,
                        Some(def_field.r#type.clone()),
                        "Struct field's type must match up with the type specified in its declaration.",
                        ),
                        continue,
                        warnings,
                        errors
                    );

                    typed_fields_buf.push(TypedStructExpressionField {
                        value: typed_field,
                        name: expr_field.name,
                    });
                }

                // check that there are no extra fields
                for field in fields {
                    if definition
                        .fields
                        .iter()
                        .find(|x| x.name == field.name)
                        .is_none()
                    {
                        errors.push(CompileError::StructDoesntHaveThisField {
                            field_name: field.name,
                            struct_name: definition.name.primary_name,
                            span: field.span,
                        });
                    }
                }
                TypedExpression {
                    expression: TypedExpressionVariant::StructExpression {
                        struct_name: definition.name.clone(),
                        fields: typed_fields_buf,
                    },
                    return_type: TypeInfo::Struct {
                        name: definition.name.clone(),
                    },
                    is_constant: IsConstant::No,
                }
            }
            Expression::SubfieldExpression { name_parts, .. } => {
                let name_parts_buf = VecDeque::from(name_parts);
                // this must be >= 2, or else the parser would not have matched it. asserting that
                // invariant here, since it is an assumption that is acted upon later.
                assert!(name_parts_buf.len() >= 2);
                todo!("subfield expressions")
            }
            Expression::MethodApplication {
                subfield_exp,
                method_name,
                arguments,
                span,
            } => {
                let (method, parent_type) = if subfield_exp.is_empty() {
                    // if subfield exp is empty, then we are calling a method using either ::
                    // syntax or an operator
                    let ns = match namespace.find_module(&method_name.prefixes) {
                        Some(o) => o,
                        None => todo!("Method not found error"),
                    };
                    // a method is defined by the type of the parent, and in this case the parent
                    // is the first argument
                    let parent_expr = match TypedExpression::type_check(
                        arguments[0].clone(),
                        namespace,
                        None,
                        "",
                    ) {
                        // throw away warnings and errors since this will be checked again later
                        CompileResult::Ok { value, .. } => value,
                        CompileResult::Err {
                            warnings: mut l_w,
                            errors: mut l_e,
                        } => {
                            warnings.append(&mut l_w);
                            errors.append(&mut l_e);
                            return err(warnings, errors);
                        }
                    };
                    (match ns.find_method_for_type(parent_expr.return_type.clone(), method_name.suffix.clone()) {
                        Some(o) => o,
                        None => {
                            errors.push(CompileError::MethodNotFound {
                                span,
                                method_name: method_name.suffix.clone().primary_name,
                                type_name: parent_expr.return_type.friendly_type_str()
                            });
                            return err(warnings, errors);

                        }
                    }, parent_expr.return_type)
                } else {
                    let parent_expr = match namespace.find_subfield(subfield_exp) {
                        Some(exp) => exp,
                        None => todo!("err, couldn't find ident"),
                    };
                    (match namespace
                        .find_method_for_type(parent_expr.return_type.clone(), method_name.suffix.clone())
                    {
                        Some(o) => o,
                        None => todo!("Method not found error"),
                    }, parent_expr.return_type)
                };

                // zip parameters to arguments to perform type checking
                let zipped = method.parameters.iter().zip(arguments.iter());

                let mut typed_arg_buf = vec![];
                for (FunctionParameter { r#type, .. }, arg) in zipped {
                    let un_self_type  = if *r#type == TypeInfo::SelfType {
                        parent_type.clone()
                    } else { r#type.clone() };
                    typed_arg_buf.push(type_check!(
                        TypedExpression::type_check(
                        arg.clone(),
                        &namespace,
                        Some(un_self_type),
                        "Function argument must be of the same type declared in the function declaration."),
                        continue, 
                        warnings,
                        errors
                    ));
                }

                TypedExpression {
                    expression: TypedExpressionVariant::FunctionApplication {
                        // TODO the prefix should be a type info maybe? and then the first arg can
                        // be self?
                        name: method_name.into(), // TODO todo!("put the actual fully-typed function bodies in these applications"),
                        arguments: typed_arg_buf,
                    },
                    return_type: method.return_type.clone(),
                    is_constant: IsConstant::No,
                }
            }
            Expression::Unit { span: _span } => TypedExpression {
                expression: TypedExpressionVariant::Unit,
                return_type: TypeInfo::Unit,
                is_constant: IsConstant::Yes,
            },

            a => {
                println!("Unimplemented semantics for expression: {:?}", a);
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
            // The annotation will result in a cast, so set the return type accordingly.
            typed_expression.return_type = type_annotation
        }

        ok(typed_expression, warnings, errors)
    }
}
