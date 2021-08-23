use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::semantic_analysis::ast_node::*;
use crate::types::{IntegerBits, MaybeResolvedType, ResolvedType};
use either::Either;

#[derive(Clone, Debug)]
pub(crate) struct TypedExpression<'sc> {
    pub(crate) expression: TypedExpressionVariant<'sc>,
    pub(crate) return_type: MaybeResolvedType<'sc>,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
    pub(crate) span: Span<'sc>,
}

pub(crate) fn error_recovery_expr<'sc>(span: Span<'sc>) -> TypedExpression<'sc> {
    TypedExpression {
        expression: TypedExpressionVariant::Unit,
        return_type: MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery),
        is_constant: IsConstant::No,
        span,
    }
}

impl<'sc> TypedExpression<'sc> {
    pub(crate) fn type_check(
        other: Expression<'sc>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<MaybeResolvedType<'sc>>,
        help_text: impl Into<String> + Clone,
        self_type: &MaybeResolvedType<'sc>,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let expr_span = other.span();
        let mut typed_expression = match other {
            Expression::Literal { value: lit, span } => {
                let return_type = match lit {
                    Literal::String(s) => {
                        MaybeResolvedType::Resolved(ResolvedType::Str(s.len() as u64))
                    }
                    Literal::U8(_) => MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::Eight,
                    )),
                    Literal::U16(_) => MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::Sixteen,
                    )),
                    Literal::U32(_) => MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::ThirtyTwo,
                    )),
                    Literal::U64(_) => MaybeResolvedType::Resolved(ResolvedType::UnsignedInteger(
                        IntegerBits::SixtyFour,
                    )),
                    Literal::Boolean(_) => MaybeResolvedType::Resolved(ResolvedType::Boolean),
                    Literal::Byte(_) => MaybeResolvedType::Resolved(ResolvedType::Byte),
                    Literal::Byte32(_) => MaybeResolvedType::Resolved(ResolvedType::Byte32),
                };
                TypedExpression {
                    expression: TypedExpressionVariant::Literal(lit),
                    return_type,
                    is_constant: IsConstant::Yes,
                    span,
                }
            }
            Expression::VariableExpression {
                name,
                unary_op,
                span,
                ..
            } => match namespace.get_symbol(&name) {
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
                    span,
                },
                Some(a) => {
                    errors.push(CompileError::NotAVariable {
                        name: name.span.as_str(),
                        span: name.span.clone(),
                        what_it_is: a.friendly_name(),
                    });
                    error_recovery_expr(name.span.clone())
                }
                None => {
                    errors.push(CompileError::UnknownVariable {
                        var_name: name.span.as_str().trim(),
                        span: name.span.clone(),
                    });
                    error_recovery_expr(name.span.clone())
                }
            },
            Expression::FunctionApplication {
                name,
                arguments,
                span,
                ..
            } => {
                let function_declaration = check!(
                    namespace.get_call_path(&name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                match function_declaration {
                    TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                        parameters,
                        return_type,
                        body,
                        is_contract_call,
                        ..
                    }) => {
                        // type check arguments in function application vs arguments in function
                        // declaration. Use parameter type annotations as annotations for the
                        // arguments
                        //
                        let typed_call_arguments = arguments
                            .into_iter()
                            .zip(parameters.iter())
                            .map(|(arg, param)| {
                                (param.name.clone(), TypedExpression::type_check(
                                    arg.clone(),
                                    namespace,
                                    Some(param.r#type.clone()),
                                    "The argument that has been provided to this function's type does \
                                    not match the declared type of the parameter in the function \
                                    declaration.",
                                    self_type,
                                    build_config,
                                    dead_code_graph,
                                )
                                .unwrap_or_else(
                                    &mut warnings,
                                    &mut errors,
                                    || error_recovery_expr(arg.span()),
                                ))
                            })
                            .collect();

                        TypedExpression {
                            return_type: return_type.clone(),
                            // now check the function call return type
                            // FEATURE this IsConstant can be true if the function itself is
                            // constant-able const functions would be an
                            // advanced feature and are not supported right
                            // now
                            is_constant: IsConstant::No,
                            expression: TypedExpressionVariant::FunctionApplication {
                                arguments: typed_call_arguments,
                                name: name.clone(),
                                function_body: body.clone(),
                                is_contract_call,
                            },
                            span,
                        }
                    }
                    a => {
                        errors.push(CompileError::NotAFunction {
                            name: name.span().as_str(),
                            span: name.span(),
                            what_it_is: a.friendly_name(),
                        });
                        error_recovery_expr(name.span())
                    }
                }
            }
            Expression::MatchExpression { span, .. } => {
                errors.push(CompileError::Unimplemented(
                    "Match expressions and pattern matching have not been implemented.",
                    span,
                ));
                return err(warnings, errors);
                /*
                let typed_primary_expression = check!(
                    TypedExpression::type_check(*primary_expression, &namespace, None, ""),
                    ERROR_RECOVERY_EXPR.clone(),
                    warnings,
                    errors
                );
                let first_branch_result = check!(
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
                            check!(
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
                */
            }
            Expression::CodeBlock { contents, span, .. } => {
                let (typed_block, block_return_type) = check!(
                    TypedCodeBlock::type_check(
                        contents.clone(),
                        &namespace,
                        type_annotation.clone(),
                        help_text.clone(),
                        self_type,
                        build_config,
                        dead_code_graph,
                    ),
                    (
                        TypedCodeBlock {
                            contents: vec![],
                            whole_block_span: span.clone()
                        },
                        Some(MaybeResolvedType::Resolved(ResolvedType::Unit))
                    ),
                    warnings,
                    errors
                );
                let block_return_type = match block_return_type {
                    Some(ty) => ty,
                    None => match type_annotation {
                        Some(ref ty) if ty != &MaybeResolvedType::Resolved(ResolvedType::Unit) => {
                            errors.push(CompileError::ExpectedImplicitReturnFromBlockWithType {
                                span: span.clone(),
                                ty: ty.friendly_type_str(),
                            });
                            MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery)
                        }
                        _ => MaybeResolvedType::Resolved(ResolvedType::Unit),
                    },
                };
                TypedExpression {
                    expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                        contents: typed_block.contents,
                        whole_block_span: span.clone(),
                    }),
                    return_type: block_return_type,
                    is_constant: IsConstant::No, /* TODO if all elements of block are constant
                                                  * then this is constant */
                    span,
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
                let condition = Box::new(check!(
                    TypedExpression::type_check(
                        *condition.clone(),
                        namespace,
                        Some(MaybeResolvedType::Resolved(ResolvedType::Boolean)),
                        "The condition of an if expression must be a boolean expression.",
                        self_type,
                        build_config,
                        dead_code_graph
                    ),
                    error_recovery_expr(condition.span()),
                    warnings,
                    errors
                ));
                let then = Box::new(check!(
                    TypedExpression::type_check(
                        *then.clone(),
                        namespace,
                        type_annotation.clone(),
                        "",
                        self_type,
                        build_config,
                        dead_code_graph
                    ),
                    error_recovery_expr(then.span()),
                    warnings,
                    errors
                ));
                let r#else = if let Some(expr) = r#else {
                    Some(Box::new(check!(
                        TypedExpression::type_check(
                            *expr.clone(),
                            namespace,
                            Some(then.return_type.clone()),
                            "",
                            self_type,
                            build_config,
                            dead_code_graph
                        ),
                        error_recovery_expr(expr.span()),
                        warnings,
                        errors
                    )))
                } else {
                    None
                };

                // if there is a type annotation, then the else branch must exist
                if let Some(ref annotation) = type_annotation {
                    if r#else.is_none() {
                        errors.push(CompileError::NoElseBranch {
                            span: span.clone(),
                            r#type: annotation.friendly_type_str(),
                        });
                    }
                }

                TypedExpression {
                    expression: TypedExpressionVariant::IfExp {
                        condition,
                        then: then.clone(),
                        r#else,
                    },
                    is_constant: IsConstant::No, // TODO
                    return_type: then.return_type,
                    span,
                }
            }
            Expression::AsmExpression { asm, span, .. } => {
                let return_type = namespace.resolve_type(&asm.return_type, self_type);
                // type check the initializers
                let typed_registers = asm
                    .registers
                    .into_iter()
                    .map(
                        |AsmRegisterDeclaration {
                             name,
                             initializer,
                             name_span,
                         }| {
                            TypedAsmRegisterDeclaration {
                                name_span: name_span.clone(),
                                name,
                                initializer: initializer.map(|initializer| {
                                    check!(
                                        TypedExpression::type_check(
                                            initializer.clone(),
                                            namespace,
                                            None,
                                            "",
                                            self_type,
                                            build_config,
                                            dead_code_graph
                                        ),
                                        error_recovery_expr(initializer.span()),
                                        warnings,
                                        errors
                                    )
                                }),
                            }
                        },
                    )
                    .collect();
                TypedExpression {
                    expression: TypedExpressionVariant::AsmExpression {
                        whole_block_span: asm.whole_block_span,
                        body: asm.body,
                        registers: typed_registers,
                        returns: asm.returns,
                    },
                    return_type,
                    is_constant: IsConstant::No,
                    span,
                }
            }
            Expression::StructExpression {
                span,
                struct_name,
                fields,
            } => {
                // TODO in here replace generic types with provided types
                // find the struct definition in the namespace
                let definition: TypedStructDeclaration =
                    match namespace.clone().get_symbol(&struct_name) {
                        Some(TypedDeclaration::StructDeclaration(st)) => st.clone(),
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
                    let expr_field: crate::parse_tree::StructExpressionField =
                        match fields.iter().find(|x| x.name == def_field.name) {
                            Some(val) => val.clone(),
                            None => {
                                errors.push(CompileError::StructMissingField {
                                    field_name: def_field.name.primary_name,
                                    struct_name: definition.name.primary_name,
                                    span: span.clone(),
                                });
                                typed_fields_buf.push(TypedStructExpressionField {
                                    name: def_field.name.clone(),
                                    value: TypedExpression {
                                        expression: TypedExpressionVariant::Unit,
                                        return_type: MaybeResolvedType::Resolved(
                                            ResolvedType::ErrorRecovery,
                                        ),
                                        is_constant: IsConstant::No,
                                        span: span.clone(),
                                    },
                                });
                                continue;
                            }
                        };

                    let typed_field = check!(
                        TypedExpression::type_check(
                            expr_field.value,
                            namespace,
                            Some(MaybeResolvedType::Resolved(def_field.r#type.clone())),
                            "Struct field's type must match up with the type specified in its \
                             declaration.",
                            self_type,
                            build_config,
                            dead_code_graph
                        ),
                        continue,
                        warnings,
                        errors
                    );

                    typed_fields_buf.push(TypedStructExpressionField {
                        value: typed_field,
                        name: expr_field.name.clone(),
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
                        errors.push(CompileError::StructDoesNotHaveField {
                            field_name: field.name.primary_name.clone(),
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
                    return_type: MaybeResolvedType::Resolved(ResolvedType::Struct {
                        name: definition.name.clone(),
                        fields: definition.fields.clone(),
                    }),
                    is_constant: IsConstant::No,
                    span,
                }
            }
            Expression::SubfieldExpression {
                unary_op,
                prefix,
                span,
                field_to_access,
            } => {
                let parent = check!(
                    TypedExpression::type_check(
                        *prefix,
                        namespace,
                        None,
                        "",
                        self_type,
                        build_config,
                        dead_code_graph
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (fields, struct_name) = check!(
                    namespace.get_struct_type_fields(
                        &parent.return_type,
                        parent.span.as_str(),
                        &parent.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );

                let field = if let Some(field) = fields
                    .iter()
                    .find(|TypedStructField { name, .. }| *name == field_to_access)
                {
                    field
                } else {
                    errors.push(CompileError::FieldNotFound {
                        span: field_to_access.span.clone(),
                        available_fields: fields
                            .iter()
                            .map(|TypedStructField { name, .. }| name.primary_name.clone())
                            .collect::<Vec<_>>()
                            .join("\n"),
                        field_name: field_to_access.primary_name,
                        struct_name: struct_name.primary_name,
                    });
                    return err(warnings, errors);
                };

                TypedExpression {
                    expression: TypedExpressionVariant::StructFieldAccess {
                        unary_op,
                        resolved_type_of_parent: parent.return_type.clone(),
                        prefix: Box::new(parent),
                        field_to_access: field.clone(),
                    },
                    return_type: MaybeResolvedType::Resolved(field.r#type.clone()),
                    is_constant: IsConstant::No,
                    span,
                }
            }
            Expression::MethodApplication {
                method_name,
                arguments,
                span,
            } => {
                match method_name {
                    // something like a.b(c)
                    MethodName::FromModule { method_name } => {
                        let mut args_buf = vec![];
                        for arg in arguments {
                            let sp = arg.span().clone();
                            args_buf.push(check!(
                                TypedExpression::type_check(
                                    arg,
                                    namespace,
                                    None,
                                    "",
                                    self_type,
                                    build_config,
                                    dead_code_graph
                                ),
                                error_recovery_expr(sp),
                                warnings,
                                errors
                            ));
                        }
                        let method = match namespace
                            .find_method_for_type(&args_buf[0].return_type, method_name.clone())
                        {
                            Some(o) => o,
                            None => {
                                if args_buf[0].return_type
                                    != MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery)
                                {
                                    errors.push(CompileError::MethodNotFound {
                                        method_name: method_name.primary_name,
                                        type_name: args_buf[0].return_type.friendly_type_str(),
                                        span: method_name.span.clone(),
                                    });
                                }
                                return err(warnings, errors);
                            }
                        };

                        // + 1 for the "self" param
                        if args_buf.len() > (method.parameters.len() + 1) {
                            errors.push(CompileError::TooManyArgumentsForFunction {
                                span: span.clone(),
                                method_name: method_name.primary_name,
                                expected: method.parameters.len(),
                                received: args_buf.len(),
                            });
                        }

                        if args_buf.len() < method.parameters.len() {
                            errors.push(CompileError::TooFewArgumentsForFunction {
                                span: span.clone(),
                                method_name: method_name.primary_name,
                                expected: method.parameters.len(),
                                received: args_buf.len(),
                            });
                        }

                        let args_and_names = method
                            .parameters
                            .iter()
                            .zip(args_buf.into_iter())
                            .map(|(param, arg)| (param.name.clone(), arg))
                            .collect::<Vec<(_, _)>>();

                        TypedExpression {
                            expression: TypedExpressionVariant::FunctionApplication {
                                name: CallPath {
                                    prefixes: vec![],
                                    suffix: method_name,
                                },
                                arguments: args_and_names,
                                function_body: method.body.clone(),
                                is_contract_call: method.is_contract_call,
                            },
                            return_type: method.return_type,
                            is_constant: IsConstant::No,
                            span,
                        }
                    }
                    // something like blah::blah::~Type::foo()
                    MethodName::FromType {
                        ref call_path,
                        ref type_name,
                        ref is_absolute,
                    } => {
                        let mut args_buf = vec![];
                        for arg in arguments {
                            args_buf.push(check!(
                                TypedExpression::type_check(
                                    arg,
                                    namespace,
                                    None,
                                    "",
                                    self_type,
                                    build_config,
                                    dead_code_graph
                                ),
                                continue,
                                warnings,
                                errors
                            ));
                        }

                        let method = if let Some(type_name) = type_name {
                            let module = check!(
                                namespace.find_module(&call_path.prefixes[..], *is_absolute),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let type_name = module.resolve_type(&type_name, self_type);
                            match module.find_method_for_type(&type_name, call_path.suffix.clone())
                            {
                                Some(o) => o,
                                None => {
                                    if type_name
                                        != MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery)
                                    {
                                        errors.push(CompileError::MethodNotFound {
                                            method_name: call_path.suffix.primary_name.clone(),
                                            type_name: type_name.friendly_type_str(),
                                            span: call_path.suffix.span.clone(),
                                        });
                                    }
                                    return err(warnings, errors);
                                }
                            }
                        } else {
                            // there is a special case for the stdlib where type_name is `None`, handle
                            // that:
                            let module = check!(
                                namespace.find_module(&call_path.prefixes[..], *is_absolute),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let r#type = &args_buf[0].return_type;
                            match module.find_method_for_type(r#type, call_path.suffix.clone()) {
                                Some(o) => o,
                                None => {
                                    if *r#type
                                        != MaybeResolvedType::Resolved(ResolvedType::ErrorRecovery)
                                    {
                                        errors.push(CompileError::MethodNotFound {
                                            method_name: call_path.suffix.primary_name.clone(),
                                            type_name: r#type.friendly_type_str(),
                                            span: call_path.suffix.span.clone(),
                                        });
                                    }
                                    return err(warnings, errors);
                                }
                            }
                        };

                        if args_buf.len() > method.parameters.len() {
                            errors.push(CompileError::TooManyArgumentsForFunction {
                                span: span.clone(),
                                method_name: method_name.easy_name(),
                                expected: method.parameters.len(),
                                received: args_buf.len(),
                            });
                        }

                        if args_buf.len() < method.parameters.len() {
                            errors.push(CompileError::TooFewArgumentsForFunction {
                                span: span.clone(),
                                method_name: method_name.easy_name(),
                                expected: method.parameters.len(),
                                received: args_buf.len(),
                            });
                        }

                        let args_and_names = method
                            .parameters
                            .iter()
                            .zip(args_buf.into_iter())
                            .map(|(param, arg)| (param.name.clone(), arg))
                            .collect::<Vec<(_, _)>>();
                        TypedExpression {
                            expression: TypedExpressionVariant::FunctionApplication {
                                name: call_path.clone(),
                                arguments: args_and_names,
                                function_body: method.body.clone(),
                                is_contract_call: method.is_contract_call,
                            },
                            return_type: method.return_type,
                            is_constant: IsConstant::No,
                            span,
                        }
                    }
                }
            }
            Expression::Unit { span } => TypedExpression {
                expression: TypedExpressionVariant::Unit,
                return_type: MaybeResolvedType::Resolved(ResolvedType::Unit),
                is_constant: IsConstant::Yes,
                span,
            },
            Expression::DelineatedPath {
                call_path,
                span,
                args,
                type_arguments,
            } => {
                // The first step is to determine if the call path refers to a module or an enum.
                // We could rely on the capitalization convention, where modules are lowercase
                // and enums are uppercase, but this is not robust in the long term.
                // Instead, we try to resolve both paths.
                // If only one exists, then we use that one. Otherwise, if both exist, it is
                // an ambiguous reference error.
                let mut probe_warnings = Vec::new();
                let mut probe_errors = Vec::new();
                let module_result = namespace
                    .find_module(&call_path.prefixes, false)
                    .ok(&mut probe_warnings, &mut probe_errors);
                let enum_module_combined_result = {
                    // also, check if this is an enum _in_ another module.
                    let (module_path, enum_name) =
                        call_path.prefixes.split_at(call_path.prefixes.len() - 1);
                    let enum_name = enum_name[0].clone();
                    let namespace = namespace.find_module(module_path, false);
                    let namespace = namespace.ok(&mut warnings, &mut errors);
                    namespace.map(|ns| ns.find_enum(&enum_name)).flatten()
                };

                let type_arguments = type_arguments
                    .iter()
                    .map(|x| namespace.resolve_type(x, self_type))
                    .collect();
                // now we can see if this thing is a symbol (typed declaration) or reference to an
                // enum instantiation
                let this_thing: Either<TypedDeclaration, TypedExpression> =
                    match (module_result, enum_module_combined_result) {
                        (Some(_module), Some(_enum_res)) => {
                            errors.push(CompileError::AmbiguousPath { span: span.clone() });
                            return err(warnings, errors);
                        }
                        (Some(module), None) => {
                            match module.get_symbol(&call_path.suffix).cloned() {
                                Some(decl) => Either::Left(decl),
                                None => {
                                    errors.push(CompileError::SymbolNotFound {
                                        name: call_path.suffix.primary_name.into(),
                                        span: call_path.suffix.span.clone(),
                                    });
                                    return err(warnings, errors);
                                }
                            }
                        }
                        (None, Some(enum_decl)) => Either::Right(check!(
                            instantiate_enum(
                                enum_decl,
                                call_path.suffix,
                                args,
                                type_arguments,
                                namespace,
                                self_type,
                                build_config,
                                dead_code_graph
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )),
                        (None, None) => {
                            errors.push(CompileError::SymbolNotFound {
                                span,
                                name: call_path.suffix.primary_name.into(),
                            });
                            return err(warnings, errors);
                        }
                    };

                match this_thing {
                    Either::Left(_) => {
                        errors.push(CompileError::Unimplemented(
                            "Unable to refer to declarations in other modules directly. Try \
                             importing it instead.",
                            span,
                        ));
                        return err(warnings, errors);
                    }
                    Either::Right(expr) => expr,
                }
            }
            Expression::AbiCast {
                abi_name,
                address,
                span,
            } => {
                // TODO use stdlib's Address type instead of byte32
                // type check the address and make sure it is
                let err_span = address.span();
                let address = check!(
                    TypedExpression::type_check(
                        *address,
                        namespace,
                        Some(MaybeResolvedType::Resolved(ResolvedType::Byte32)),
                        "An address that is being ABI cast must be of type byte32",
                        self_type,
                        build_config,
                        dead_code_graph,
                    ),
                    error_recovery_expr(err_span),
                    warnings,
                    errors
                );
                // look up the call path and get the declaration it references
                let abi = check!(
                    namespace.get_call_path(&abi_name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // make sure the declaration is actually an abi
                let abi = match abi {
                    TypedDeclaration::AbiDeclaration(abi) => abi,
                    a => {
                        errors.push(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: a.friendly_name(),
                        });
                        return err(warnings, errors);
                    }
                };
                let return_type =
                    MaybeResolvedType::Resolved(ResolvedType::ContractCaller(abi_name.clone()));
                let mut functions_buf = abi
                    .interface_surface
                    .iter()
                    .map(|x| x.to_dummy_func(Mode::ImplAbiFn))
                    .collect::<Vec<_>>();
                functions_buf.append(&mut abi.methods.clone());
                namespace.insert_trait_implementation(
                    abi_name.clone(),
                    return_type.clone(),
                    functions_buf,
                );
                // send some out-of-band flag that these are contract calls so the code
                // generation knows what's up
                TypedExpression {
                    expression: TypedExpressionVariant::AbiCast {
                        abi_name,
                        address: Box::new(address),
                        span: span.clone(),
                        abi,
                    },
                    return_type,
                    is_constant: IsConstant::No,
                    span,
                }
            }
            a => {
                println!("Unimplemented semantic_analysis for expression: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                ));

                error_recovery_expr(a.span())
            }
        };
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            let convertability = typed_expression.return_type.is_convertible(
                &type_annotation,
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
    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} ({})",
            self.expression.pretty_print(),
            self.return_type.friendly_type_str()
        )
    }
}
