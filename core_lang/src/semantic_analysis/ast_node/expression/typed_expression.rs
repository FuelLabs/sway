use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::semantic_analysis::ast_node::*;
use crate::type_engine::TypeEngine;
use crate::types::{IntegerBits, MaybeResolvedType, ResolvedType};
use either::Either;

mod method_application;
use crate::type_engine::{Engine, TypeId};
use method_application::type_check_method_application;

#[derive(Clone, Debug)]
pub struct TypedExpression<'sc> {
    pub(crate) expression: TypedExpressionVariant<'sc>,
    pub(crate) return_type: TypeId,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
    pub(crate) span: Span<'sc>,
}

pub(crate) fn error_recovery_expr<'sc>(span: Span<'sc>) -> TypedExpression<'sc> {
    TypedExpression {
        expression: TypedExpressionVariant::Unit,
        return_type: todo!("reserved error recovery type id"),
        is_constant: IsConstant::No,
        span,
    }
}

impl<'sc> TypedExpression<'sc> {
    pub(crate) fn type_check(
        other: Expression<'sc>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, Self> {
        let engine: crate::type_engine::Engine = todo!("global engine");
        let expr_span = other.span();
        let res = match other {
            Expression::Literal { value: lit, span } => {
                Self::type_check_literal(lit, span, namespace)
            }
            Expression::VariableExpression { name, span, .. } => {
                Self::type_check_variable_expression(name, span, namespace)
            }
            Expression::FunctionApplication {
                name,
                arguments,
                span,
                ..
            } => Self::type_check_function_application(
                name,
                arguments,
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::LazyOperator { op, lhs, rhs, span } => Self::type_check_lazy_operator(
                op,
                *lhs,
                *rhs,
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::MatchExpression { span, .. } => {
                let errors = vec![CompileError::Unimplemented(
                    "Match expressions and pattern matching have not been implemented.",
                    span,
                )];
                return err(vec![], errors);
            }
            Expression::CodeBlock { contents, span, .. } => Self::type_check_code_block(
                contents,
                span,
                namespace,
                type_annotation,
                help_text.clone(),
                self_type,
                build_config,
                dead_code_graph,
            ),
            // TODO if _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch
            Expression::IfExp {
                condition,
                then,
                r#else,
                span,
            } => Self::type_check_if_expression(
                condition,
                then,
                r#else,
                span,
                namespace,
                type_annotation.clone(),
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::AsmExpression { asm, span, .. } => Self::type_check_asm_expression(
                asm,
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::StructExpression {
                span,
                struct_name,
                fields,
            } => Self::type_check_struct_expression(
                span,
                struct_name,
                fields,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::SubfieldExpression {
                prefix,
                span,
                field_to_access,
            } => Self::type_check_subfield_expression(
                prefix,
                span,
                field_to_access,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::MethodApplication {
                method_name,
                arguments,
                span,
            } => type_check_method_application(
                method_name,
                arguments,
                span.clone(),
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::Unit { span } => {
                let exp = TypedExpression {
                    expression: TypedExpressionVariant::Unit,
                    return_type: todo!("unit type in engine -- maybe reserved?"),
                    is_constant: IsConstant::Yes,
                    span,
                };
                ok(exp, vec![], vec![])
            }
            Expression::DelineatedPath {
                call_path,
                span,
                args,
                type_arguments,
            } => Self::type_check_delineated_path(
                call_path,
                span,
                args,
                type_arguments,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            Expression::AbiCast {
                abi_name,
                address,
                span,
            } => Self::type_check_abi_cast(
                abi_name,
                address,
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
            ),
            a => {
                let mut errors = vec![];
                println!("Unimplemented semantic_analysis for expression: {:?}", a);
                errors.push(CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                ));

                let exp = error_recovery_expr(a.span());
                ok(exp, vec![], errors)
            }
        };
        let mut typed_expression = match res.value {
            Some(r) => r,
            None => return res,
        };
        let mut warnings = res.warnings;
        let mut errors = res.errors;
        // if the return type cannot be cast into the annotation type then it is a type error
        if let Some(type_annotation) = type_annotation {
            match engine.unify_with_self(
                typed_expression.return_type,
                type_annotation,
                self_type,
                &expr_span,
            ) {
                Ok(warning) => {
                    if let Some(warning) = warning {
                        warnings.push(CompileWarning {
                            warning_content: warning,
                            span: expr_span,
                        });
                    }
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            };
            // The annotation may result in a cast, which is handled in the type engine.
        }

        ok(typed_expression, warnings, errors)
    }

    fn type_check_literal(
        lit: Literal<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let return_type = match lit {
            Literal::String(s) => TypeInfo::Str(s.len() as u64),
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),

            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::Byte(_) => TypeInfo::Byte,
            Literal::B256(_) => TypeInfo::B256,
        };
        let engine: crate::type_engine::Engine = todo!("global engine");
        let id = engine.insert(return_type);
        let exp = TypedExpression {
            expression: TypedExpressionVariant::Literal(lit),
            return_type: id,
            is_constant: IsConstant::Yes,
            span,
        };
        ok(exp, vec![], vec![])
    }

    fn type_check_variable_expression(
        name: Ident<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut errors = vec![];
        let exp = match namespace.get_symbol(&name) {
            Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                body, ..
            })) => TypedExpression {
                return_type: body.return_type.clone(),
                is_constant: body.is_constant,
                expression: TypedExpressionVariant::VariableExpression { name: name.clone() },
                span,
            },
            Some(TypedDeclaration::ConstantDeclaration(TypedConstantDeclaration {
                value, ..
            })) => TypedExpression {
                return_type: value.return_type.clone(),
                is_constant: IsConstant::Yes,
                // Although this isn't strictly a 'variable' expression we can treat it as one for
                // this context.
                expression: TypedExpressionVariant::VariableExpression { name: name.clone() },
                span,
            },
            Some(a) => {
                errors.push(CompileError::NotAVariable {
                    name: name.span.as_str().to_string(),
                    span: name.span.clone(),
                    what_it_is: a.friendly_name(),
                });
                error_recovery_expr(name.span.clone())
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.span.as_str().trim().to_string(),
                    span: name.span.clone(),
                });
                error_recovery_expr(name.span.clone())
            }
        };
        ok(exp, vec![], errors)
    }

    fn type_check_function_application(
        name: CallPath<'sc>,
        arguments: Vec<Expression<'sc>>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let function_declaration = check!(
            namespace.get_call_path(&name),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = match function_declaration {
            TypedDeclaration::FunctionDeclaration(decl) => {
                let TypedFunctionDeclaration {
                    parameters,
                    return_type,
                    body,
                    ..
                } = decl.clone();
                if arguments.len() > parameters.len() {
                    let arguments_span = arguments.iter().fold(
                        arguments
                            .get(0)
                            .map(|x| x.span())
                            .unwrap_or_else(|| name.span()),
                        |acc, arg| crate::utils::join_spans(acc, arg.span()),
                    );
                    errors.push(CompileError::TooManyArgumentsForFunction {
                        span: arguments_span,
                        method_name: name.suffix.primary_name,
                        expected: parameters.len(),
                        received: arguments.len(),
                    });
                } else if arguments.len() < parameters.len() {
                    let arguments_span = arguments.iter().fold(
                        arguments
                            .get(0)
                            .map(|x| x.span())
                            .unwrap_or_else(|| name.span()),
                        |acc, arg| crate::utils::join_spans(acc, arg.span()),
                    );
                    errors.push(CompileError::TooFewArgumentsForFunction {
                        span: arguments_span,
                        method_name: name.suffix.primary_name,
                        expected: parameters.len(),
                        received: arguments.len(),
                    });
                }
                // type check arguments in function application vs arguments in function
                // declaration. Use parameter type annotations as annotations for the
                // arguments
                //
                let typed_call_arguments =
                    arguments
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
                        selector: None, // regular functions cannot be in a contract call; only methods
                    },
                    span,
                }
            }
            a => {
                errors.push(CompileError::NotAFunction {
                    name: name.span().as_str().to_string(),
                    span: name.span(),
                    what_it_is: a.friendly_name(),
                });
                error_recovery_expr(name.span())
            }
        };
        ok(exp, warnings, errors)
    }

    fn type_check_lazy_operator(
        op: LazyOp,
        lhs: Expression<'sc>,
        rhs: Expression<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let engine: crate::type_engine::Engine = todo!("engine");
        let mut warnings = vec![];
        let mut errors = vec![];
        let bool_type_id = engine.insert(TypeInfo::Boolean);
        let typed_lhs = check!(
            TypedExpression::type_check(
                lhs.clone(),
                namespace,
                Some(bool_type_id),
                "",
                self_type,
                build_config,
                dead_code_graph
            ),
            error_recovery_expr(lhs.span()),
            warnings,
            errors
        );

        let typed_rhs = check!(
            TypedExpression::type_check(
                rhs.clone(),
                namespace,
                Some(bool_type_id),
                "",
                self_type,
                build_config,
                dead_code_graph
            ),
            error_recovery_expr(rhs.span()),
            warnings,
            errors
        );

        ok(
            TypedExpression {
                expression: TypedExpressionVariant::LazyOperator {
                    op,
                    lhs: Box::new(typed_lhs),
                    rhs: Box::new(typed_rhs),
                },
                return_type: bool_type_id,
                is_constant: IsConstant::No, // Maybe.
                span,
            },
            warnings,
            errors,
        )
    }

    pub fn type_check_match_expression() -> CompileResult<'sc, TypedExpression<'sc>> {
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
        unimplemented!()
    }

    fn type_check_code_block(
        contents: CodeBlock<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut engine: crate::type_engine::Engine = todo!("global engine");
        let (typed_block, block_return_type) = check!(
            TypedCodeBlock::type_check(
                contents.clone(),
                namespace,
                type_annotation.unwrap_or_else(|| engine.insert(TypeInfo::Unknown)),
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
                engine.insert(TypeInfo::Unit)
            ),
            warnings,
            errors
        );
        let block_return_type: TypeId = match namespace.look_up_type_id(block_return_type) {
            ResolvedType::Unit => match type_annotation {
                Some(ref ty) if namespace.look_up_type_id(*ty) != ResolvedType::Unit => {
                    errors.push(CompileError::ExpectedImplicitReturnFromBlockWithType {
                        span: span.clone(),
                        ty: namespace.look_up_type_id(*ty).friendly_type_str(),
                    });
                    engine.insert(TypeInfo::ErrorRecovery)
                }
                _ => engine.insert(TypeInfo::Unit),
            },
            otherwise => block_return_type,
        };
        let exp = TypedExpression {
            expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                contents: typed_block.contents,
                whole_block_span: span.clone(),
            }),
            return_type: block_return_type,
            is_constant: IsConstant::No, /* TODO if all elements of block are constant
                                          * then this is constant */
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_if_expression(
        condition: Box<Expression<'sc>>,
        then: Box<Expression<'sc>>,
        r#else: Option<Box<Expression<'sc>>>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        type_annotation: Option<TypeId>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let engine: crate::type_engine::Engine = todo!("global engine");
        let mut errors = vec![];
        let condition = Box::new(check!(
            TypedExpression::type_check(
                *condition.clone(),
                namespace,
                Some(engine.insert(TypeInfo::Boolean)),
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
                    r#type: namespace.look_up_type_id(*annotation).friendly_type_str(),
                });
            }
        }

        let exp = TypedExpression {
            expression: TypedExpressionVariant::IfExp {
                condition,
                then: then.clone(),
                r#else,
            },
            is_constant: IsConstant::No, // TODO
            return_type: then.return_type,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_asm_expression(
        asm: AsmExpression<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let return_type = namespace.resolve_type_with_self(asm.return_type, self_type);
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
        let exp = TypedExpression {
            expression: TypedExpressionVariant::AsmExpression {
                whole_block_span: asm.whole_block_span,
                body: asm.body,
                registers: typed_registers,
                returns: asm.returns,
            },
            return_type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_struct_expression(
        span: Span<'sc>,
        struct_name: Ident<'sc>,
        fields: Vec<StructExpressionField<'sc>>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut typed_fields_buf = vec![];

        // TODO in here replace generic types with provided types
        // find the struct definition in the namespace
        let definition: TypedStructDeclaration = match namespace.clone().get_symbol(&struct_name) {
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

        let engine: crate::type_engine::Engine = todo!("global engine");
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
                                return_type: engine.insert(TypeInfo::ErrorRecovery),
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
                    Some(def_field.r#type),
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
            if !definition.fields.iter().any(|x| x.name == field.name) {
                errors.push(CompileError::StructDoesNotHaveField {
                    field_name: &(*field.name.primary_name),
                    struct_name: definition.name.primary_name,
                    span: field.span,
                });
            }
        }
        let engine: crate::type_engine::Engine = todo!("global engine");
        let struct_type_id = engine.insert(TypeInfo::Struct {
            name: definition.name.clone(),
            fields: definition.fields.clone(),
        });
        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructExpression {
                struct_name: definition.name.clone(),
                fields: typed_fields_buf,
            },
            return_type: struct_type_id,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_subfield_expression(
        prefix: Box<Expression<'sc>>,
        span: Span<'sc>,
        field_to_access: Ident<'sc>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
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
                parent.return_type,
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
                    .map(|TypedStructField { name, .. }| &(*name.primary_name))
                    .collect::<Vec<_>>()
                    .join("\n"),
                field_name: field_to_access.primary_name,
                struct_name: struct_name.primary_name,
            });
            return err(warnings, errors);
        };

        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent: parent.return_type.clone(),
                prefix: Box::new(parent),
                field_to_access: field.clone(),
            },
            return_type: field.r#type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_delineated_path(
        call_path: CallPath<'sc>,
        span: Span<'sc>,
        args: Vec<Expression<'sc>>,
        type_arguments: Vec<TypeInfo<'sc>>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
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

        let engine: crate::type_engine::Engine = todo!("global engine");
        // now we can see if this thing is a symbol (typed declaration) or reference to an
        // enum instantiation
        let this_thing: Either<TypedDeclaration, TypedExpression> =
            match (module_result, enum_module_combined_result) {
                (Some(_module), Some(_enum_res)) => {
                    errors.push(CompileError::AmbiguousPath { span: span.clone() });
                    return err(warnings, errors);
                }
                (Some(module), None) => match module.get_symbol(&call_path.suffix).cloned() {
                    Some(decl) => Either::Left(decl),
                    None => {
                        errors.push(CompileError::SymbolNotFound {
                            name: call_path.suffix.primary_name,
                            span: call_path.suffix.span.clone(),
                        });
                        return err(warnings, errors);
                    }
                },
                (None, Some(enum_decl)) => Either::Right(check!(
                    instantiate_enum(
                        enum_decl,
                        call_path.suffix,
                        args,
                        todo!("generics"),
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
                        name: call_path.suffix.primary_name,
                    });
                    return err(warnings, errors);
                }
            };

        let exp = match this_thing {
            Either::Left(_) => {
                errors.push(CompileError::Unimplemented(
                    "Unable to refer to declarations in other modules directly. Try \
                     importing it instead.",
                    span,
                ));
                return err(warnings, errors);
            }
            Either::Right(expr) => expr,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_abi_cast(
        abi_name: CallPath<'sc>,
        address: Box<Expression<'sc>>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
    ) -> CompileResult<'sc, TypedExpression<'sc>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // TODO use stdlib's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span();
        let engine: crate::type_engine::Engine = todo!("global engine");
        let address = check!(
            TypedExpression::type_check(
                *address,
                namespace,
                Some(engine.insert(TypeInfo::B256)),
                "An address that is being ABI cast must be of type b256",
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
        let return_type = engine.insert(TypeInfo::ContractCaller {
            abi_name: abi_name.clone(),
            address: Box::new(address.clone()),
        });
        let mut functions_buf = abi
            .interface_surface
            .iter()
            .map(|x| x.to_dummy_func(Mode::ImplAbiFn))
            .collect::<Vec<_>>();
        // calls of ABI methods do not result in any codegen of the ABI method block
        // they instead just use the CALL opcode and the return type
        let mut type_checked_fn_buf = Vec::with_capacity(abi.methods.len());
        for method in &abi.methods {
            type_checked_fn_buf.push(check!(
                TypedFunctionDeclaration::type_check(
                    method.clone(),
                    namespace,
                    engine.insert(TypeInfo::Unknown),
                    "",
                    engine.insert(TypeInfo::Contract),
                    build_config,
                    dead_code_graph,
                    Mode::ImplAbiFn
                ),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        functions_buf.append(&mut type_checked_fn_buf);
        namespace.insert_trait_implementation(
            abi_name.clone(),
            namespace.look_up_type_id(return_type.clone()),
            functions_buf,
        );
        let exp = TypedExpression {
            expression: TypedExpressionVariant::AbiCast {
                abi_name,
                address: Box::new(address),
                span: span.clone(),
                abi,
            },
            return_type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} ({})",
            self.expression.pretty_print(todo!("global engine")),
            todo!("lazy static type engine") //engine.look_up_type_id(self.return_type).friendly_type_str()
        )
    }
}
