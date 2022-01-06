use super::*;
use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::semantic_analysis::{ast_node::*, Namespace, TypeCheckArguments};
use crate::type_engine::{insert_type, IntegerBits};

use either::Either;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

mod method_application;
use crate::type_engine::TypeId;
use method_application::type_check_method_application;

#[derive(Clone, Debug)]
pub struct TypedExpression {
    pub(crate) expression: TypedExpressionVariant,
    pub(crate) return_type: TypeId,
    /// whether or not this expression is constantly evaluatable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
    pub(crate) span: Span,
}

pub(crate) fn error_recovery_expr(span: Span) -> TypedExpression {
    TypedExpression {
        expression: TypedExpressionVariant::Tuple { fields: vec![] },
        return_type: crate::type_engine::insert_type(TypeInfo::ErrorRecovery),
        is_constant: IsConstant::No,
        span,
    }
}

#[allow(clippy::too_many_arguments)]
impl TypedExpression {
    pub(crate) fn type_check(arguments: TypeCheckArguments<'_, Expression>) -> CompileResult<Self> {
        let TypeCheckArguments {
            checkee: other,
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            help_text,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            opts,
            ..
        } = arguments;
        let expr_span = other.span();
        let res = match other {
            Expression::Literal { value: lit, span } => Self::type_check_literal(lit, span),
            Expression::VariableExpression { name, span, .. } => {
                Self::type_check_variable_expression(name, span, namespace)
            }
            Expression::FunctionApplication {
                name,
                arguments,
                span,
                type_arguments,
                ..
            } => Self::type_check_function_application(
                TypeCheckArguments {
                    checkee: (name, arguments, type_arguments),
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                },
                span,
            ),
            Expression::LazyOperator { op, lhs, rhs, span } => Self::type_check_lazy_operator(
                TypeCheckArguments {
                    checkee: (op, *lhs, *rhs),
                    return_type_annotation: insert_type(TypeInfo::Boolean),
                    namespace,
                    crate_namespace,
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                },
                span,
            ),
            Expression::CodeBlock { contents, span, .. } => Self::type_check_code_block(
                contents,
                span,
                namespace,
                crate_namespace,
                type_annotation,
                help_text,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
            // TODO if _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch
            Expression::IfExp {
                condition,
                then,
                r#else,
                span,
            } => Self::type_check_if_expression(
                TypeCheckArguments {
                    checkee: (condition, then, r#else),
                    return_type_annotation: type_annotation,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    help_text: Default::default(),
                    opts,
                },
                span,
            ),
            Expression::AsmExpression { asm, span, .. } => Self::type_check_asm_expression(
                asm,
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
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
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
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
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
            Expression::MethodApplication {
                method_name,
                arguments,
                span,
            } => type_check_method_application(
                method_name,
                arguments,
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
            Expression::Tuple { fields, span } => Self::type_check_tuple(
                fields,
                span,
                namespace,
                crate_namespace,
                type_annotation,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
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
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
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
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
            Expression::Array { contents, span } => Self::type_check_array(
                contents,
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            ),
            Expression::ArrayIndex {
                prefix,
                index,
                span,
            } => Self::type_check_array_index(
                TypeCheckArguments {
                    checkee: (*prefix, *index),
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    opts,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    mode: Default::default(),
                    help_text: Default::default(),
                },
                span,
            ),
            Expression::DelayedMatchTypeResolution { variant, span } => {
                Self::type_check_delayed_resolution(
                    variant,
                    span,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    opts,
                )
            }
            a => {
                let errors = vec![CompileError::Unimplemented(
                    "Unimplemented expression",
                    a.span(),
                )];

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
        match unify_with_self(
            typed_expression.return_type,
            type_annotation,
            self_type,
            &expr_span,
        ) {
            Ok(mut ws) => {
                warnings.append(&mut ws);
            }
            Err(e) => {
                errors.push(CompileError::TypeError(e));
            }
        };
        // The annotation may result in a cast, which is handled in the type engine.

        typed_expression.return_type = namespace
            .resolve_type_with_self(look_up_type_id(typed_expression.return_type), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType { span: expr_span });
                insert_type(TypeInfo::ErrorRecovery)
            });

        ok(typed_expression, warnings, errors)
    }

    /// Makes a fresh copy of all type ids in this expression. Used when monomorphizing.
    pub(crate) fn copy_types(&mut self, type_mapping: &[(TypeParameter, TypeId)]) {
        self.return_type = if let Some(matching_id) =
            look_up_type_id(self.return_type).matches_type_parameter(type_mapping)
        {
            insert_type(TypeInfo::Ref(matching_id))
        } else {
            insert_type(look_up_type_id_raw(self.return_type))
        };

        self.expression.copy_types(type_mapping);
    }

    fn type_check_literal(lit: Literal, span: Span) -> CompileResult<TypedExpression> {
        let return_type = match &lit {
            Literal::String(s) => TypeInfo::Str(s.as_str().len() as u64),
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),

            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::Byte(_) => TypeInfo::Byte,
            Literal::B256(_) => TypeInfo::B256,
        };
        let id = crate::type_engine::insert_type(return_type);
        let exp = TypedExpression {
            expression: TypedExpressionVariant::Literal(lit),
            return_type: id,
            is_constant: IsConstant::Yes,
            span,
        };
        ok(exp, vec![], vec![])
    }

    fn type_check_variable_expression(
        name: Ident,
        span: Span,
        namespace: &Namespace,
    ) -> CompileResult<TypedExpression> {
        let mut errors = vec![];
        let exp = match namespace.get_symbol(&name).value {
            Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                body, ..
            })) => TypedExpression {
                return_type: body.return_type,
                is_constant: body.is_constant,
                expression: TypedExpressionVariant::VariableExpression { name: name.clone() },
                span,
            },
            Some(TypedDeclaration::ConstantDeclaration(TypedConstantDeclaration {
                value, ..
            })) => TypedExpression {
                return_type: value.return_type,
                is_constant: IsConstant::Yes,
                // Although this isn't strictly a 'variable' expression we can treat it as one for
                // this context.
                expression: TypedExpressionVariant::VariableExpression { name: name.clone() },
                span,
            },
            Some(a) => {
                errors.push(CompileError::NotAVariable {
                    name: name.span().as_str().to_string(),
                    span: name.span().clone(),
                    what_it_is: a.friendly_name(),
                });
                error_recovery_expr(name.span().clone())
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.span().as_str().trim().to_string(),
                    span: name.span().clone(),
                });
                error_recovery_expr(name.span().clone())
            }
        };
        ok(exp, vec![], errors)
    }

    #[allow(clippy::type_complexity)]
    fn type_check_function_application(
        arguments: TypeCheckArguments<'_, (CallPath, Vec<Expression>, Vec<(TypeInfo, Span)>)>,
        _span: Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee: (name, arguments, type_arguments),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            opts,
            ..
        } = arguments;
        let mut warnings = vec![];
        let mut errors = vec![];
        let function_declaration = check!(
            namespace.get_call_path(&name),
            return err(warnings, errors),
            warnings,
            errors
        );
        let TypedFunctionDeclaration {
            parameters,
            return_type,
            body,
            span,
            purity,
            ..
        } = if let TypedDeclaration::FunctionDeclaration(decl) = function_declaration {
            // if this is a generic function, monomorphize its internal types and insert the resulting
            // declaration into the namespace. Then, use that instead.
            if decl.type_parameters.is_empty() {
                decl
            } else {
                check!(
                    decl.monomorphize(type_arguments, self_type),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
        } else {
            errors.push(CompileError::NotAFunction {
                name: name.span().as_str().to_string(),
                span: name.span(),
                what_it_is: function_declaration.friendly_name(),
            });
            return err(warnings, errors);
        };

        if opts.purity != purity {
            errors.push(CompileError::PureCalledImpure { span: name.span() });
        }

        match arguments.len().cmp(&parameters.len()) {
            Ordering::Greater => {
                let arguments_span = arguments.iter().fold(
                    arguments
                        .get(0)
                        .map(|x| x.span())
                        .unwrap_or_else(|| name.span()),
                    |acc, arg| crate::utils::join_spans(acc, arg.span()),
                );
                errors.push(CompileError::TooManyArgumentsForFunction {
                    span: arguments_span,
                    method_name: name.suffix.clone(),
                    expected: parameters.len(),
                    received: arguments.len(),
                });
            }
            Ordering::Less => {
                let arguments_span = arguments.iter().fold(
                    arguments
                        .get(0)
                        .map(|x| x.span())
                        .unwrap_or_else(|| name.span()),
                    |acc, arg| crate::utils::join_spans(acc, arg.span()),
                );
                errors.push(CompileError::TooFewArgumentsForFunction {
                    span: arguments_span,
                    method_name: name.suffix.clone(),
                    expected: parameters.len(),
                    received: arguments.len(),
                });
            }
            Ordering::Equal => {}
        }
        // type check arguments in function application vs arguments in function
        // declaration. Use parameter type annotations as annotations for the
        // arguments
        //
        let typed_call_arguments = arguments
            .into_iter()
            .zip(parameters.iter())
            .map(|(arg, param)| {
                (
                    param.name.clone(),
                    TypedExpression::type_check(TypeCheckArguments {
                        checkee: arg.clone(),
                        namespace,
                        crate_namespace,
                        return_type_annotation: param.r#type,
                        help_text:
                            "The argument that has been provided to this function's type does \
                            not match the declared type of the parameter in the function \
                            declaration.",
                        self_type,
                        build_config,
                        dead_code_graph,
                        dependency_graph,
                        mode: Mode::NonAbi,
                        opts,
                    })
                    .unwrap_or_else(&mut warnings, &mut errors, || {
                        error_recovery_expr(arg.span())
                    }),
                )
            })
            .collect();

        ok(
            TypedExpression {
                return_type,
                // now check the function call return type
                // FEATURE this IsConstant can be true if the function itself is
                // constant-able const functions would be an
                // advanced feature and are not supported right
                // now
                is_constant: IsConstant::No,
                expression: TypedExpressionVariant::FunctionApplication {
                    arguments: typed_call_arguments,
                    name,
                    function_body: body,
                    selector: None, // regular functions cannot be in a contract call; only methods
                },
                span,
            },
            warnings,
            errors,
        )
    }

    fn type_check_lazy_operator(
        arguments: TypeCheckArguments<'_, (LazyOp, Expression, Expression)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee: (op, lhs, rhs),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            return_type_annotation,
            opts,
            ..
        } = arguments;

        let mut warnings = vec![];
        let mut errors = vec![];
        let typed_lhs = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: lhs.clone(),
                help_text: Default::default(),
                mode: Mode::NonAbi,
                opts,
                self_type,
                namespace,
                crate_namespace,
                return_type_annotation,
                build_config,
                dead_code_graph,
                dependency_graph,
            }),
            error_recovery_expr(lhs.span()),
            warnings,
            errors
        );

        let typed_rhs = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: rhs.clone(),
                namespace,
                crate_namespace,
                return_type_annotation,
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
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
                return_type: return_type_annotation,
                is_constant: IsConstant::No, // Maybe.
                span,
            },
            warnings,
            errors,
        )
    }

    fn type_check_code_block<'n>(
        contents: CodeBlock,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        type_annotation: TypeId,
        help_text: &'static str,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (typed_block, block_return_type) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: contents,
                namespace,
                crate_namespace,
                return_type_annotation: type_annotation,
                help_text,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            (
                TypedCodeBlock {
                    contents: vec![],
                    whole_block_span: span.clone()
                },
                crate::type_engine::insert_type(TypeInfo::Tuple(Vec::new()))
            ),
            warnings,
            errors
        );

        // this could probably be cleaned up with unification instead of comparing types
        match unify_with_self(block_return_type, type_annotation, self_type, &span) {
            Ok(mut ws) => {
                warnings.append(&mut ws);
            }
            Err(e) => {
                errors.push(e.into());
            }
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

    #[allow(clippy::type_complexity)]
    fn type_check_if_expression(
        arguments: TypeCheckArguments<
            '_,
            (Box<Expression>, Box<Expression>, Option<Box<Expression>>),
        >,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee: (condition, then, r#else),
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            opts,
            ..
        } = arguments;
        let mut warnings = vec![];
        let mut errors = vec![];
        let condition = Box::new(check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *condition.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Boolean),
                help_text: "The condition of an if expression must be a boolean expression.",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(condition.span()),
            warnings,
            errors
        ));
        let then = Box::new(check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *then.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: type_annotation,
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(then.span()),
            warnings,
            errors
        ));
        let r#else = r#else.map(|expr| {
            Box::new(check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: *expr.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: then.return_type,
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                error_recovery_expr(expr.span()),
                warnings,
                errors
            ))
        });

        let r#else_ret_ty = r#else
            .as_ref()
            .map(|x| x.return_type)
            .unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new())));
        // if there is a type annotation, then the else branch must exist
        match unify_with_self(then.return_type, r#else_ret_ty, self_type, &span) {
            Ok(mut warn) => {
                warnings.append(&mut warn);
                if !look_up_type_id(r#else_ret_ty).is_unit() && r#else.is_none() {
                    errors.push(CompileError::NoElseBranch {
                        span: span.clone(),
                        r#type: look_up_type_id(type_annotation).friendly_type_str(),
                    });
                }
            }
            Err(e) => {
                errors.push(e.into());
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

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression<'n>(
        asm: AsmExpression,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let return_type = namespace
            .resolve_type_with_self(asm.return_type.clone(), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType {
                    span: asm
                        .returns
                        .clone()
                        .map(|x| x.1)
                        .unwrap_or_else(|| asm.whole_block_span.clone()),
                });
                insert_type(TypeInfo::ErrorRecovery)
            });
        // type check the initializers
        let typed_registers = asm
            .registers
            .into_iter()
            .map(
                |AsmRegisterDeclaration { name, initializer }| TypedAsmRegisterDeclaration {
                    name,
                    initializer: initializer.map(|initializer| {
                        check!(
                            TypedExpression::type_check(TypeCheckArguments {
                                checkee: initializer.clone(),
                                namespace,
                                crate_namespace,
                                return_type_annotation: insert_type(TypeInfo::Unknown),
                                help_text: Default::default(),
                                self_type,
                                build_config,
                                dead_code_graph,
                                dependency_graph,
                                mode: Mode::NonAbi,
                                opts,
                            }),
                            error_recovery_expr(initializer.span()),
                            warnings,
                            errors
                        )
                    }),
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

    #[allow(clippy::too_many_arguments)]
    fn type_check_struct_expression<'n>(
        span: Span,
        struct_name: Ident,
        fields: Vec<StructExpressionField>,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut typed_fields_buf = vec![];

        let definition: TypedStructDeclaration =
            match namespace.clone().get_symbol(&struct_name).value {
                Some(TypedDeclaration::StructDeclaration(st)) => st.clone(),
                Some(_) => {
                    errors.push(CompileError::DeclaredNonStructAsStruct {
                        name: struct_name.clone(),
                        span,
                    });
                    return err(warnings, errors);
                }
                None => {
                    errors.push(CompileError::StructNotFound {
                        name: struct_name.clone(),
                        span,
                    });
                    return err(warnings, errors);
                }
            };
        // if this is a generic struct, i.e. it has some type
        // parameters, monomorphize it before unifying the
        // types
        let definition = if definition.type_parameters.is_empty() {
            definition
        } else {
            definition.monomorphize()
        };

        // match up the names with their type annotations from the declaration
        for def_field in definition.fields.iter() {
            let expr_field: crate::parse_tree::StructExpressionField =
                match fields.iter().find(|x| x.name == def_field.name) {
                    Some(val) => val.clone(),
                    None => {
                        errors.push(CompileError::StructMissingField {
                            field_name: def_field.name.clone(),
                            struct_name: definition.name.clone(),
                            span: span.clone(),
                        });
                        typed_fields_buf.push(TypedStructExpressionField {
                            name: def_field.name.clone(),
                            value: TypedExpression {
                                expression: TypedExpressionVariant::Tuple { fields: vec![] },
                                return_type: insert_type(TypeInfo::ErrorRecovery),
                                is_constant: IsConstant::No,
                                span: span.clone(),
                            },
                        });
                        continue;
                    }
                };

            let typed_field = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: expr_field.value,
                    namespace,
                    crate_namespace,
                    return_type_annotation: def_field.r#type,
                    help_text: "Struct field's type must match up with the type specified in its \
                     declaration.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
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
                    field_name: field.name.clone(),
                    struct_name: definition.name.clone(),
                    span: field.span,
                });
            }
        }
        let struct_type_id = crate::type_engine::insert_type(TypeInfo::Struct {
            name: definition.name.as_str().to_string(),
            fields: definition
                .fields
                .iter()
                .map(TypedStructField::as_owned_typed_struct_field)
                .collect::<Vec<_>>(),
        });
        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructExpression {
                struct_name: definition.name,
                fields: typed_fields_buf,
            },
            return_type: struct_type_id,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_subfield_expression<'n>(
        prefix: Box<Expression>,
        span: Span,
        field_to_access: Ident,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let parent = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *prefix,
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
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
            .find(|OwnedTypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            field
        } else {
            errors.push(CompileError::FieldNotFound {
                span: field_to_access.span().clone(),
                available_fields: fields
                    .iter()
                    .map(|OwnedTypedStructField { name, .. }| name.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                field_name: field_to_access.clone(),
                struct_name,
            });
            return err(warnings, errors);
        };

        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent: parent.return_type,
                prefix: Box::new(parent),
                field_to_access: field.clone(),
                field_to_access_span: span.clone(),
            },
            return_type: field.r#type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_tuple<'n>(
        fields: Vec<Expression>,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        type_annotation: TypeId,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let field_type_ids_opt = match look_up_type_id(type_annotation) {
            TypeInfo::Tuple(field_type_ids) if field_type_ids.len() == fields.len() => {
                Some(field_type_ids)
            }
            _ => None,
        };
        let mut typed_field_types = Vec::with_capacity(fields.len());
        let mut typed_fields = Vec::with_capacity(fields.len());
        let mut is_constant = IsConstant::Yes;
        for (i, field) in fields.into_iter().enumerate() {
            let field_type_id = field_type_ids_opt
                .as_ref()
                .map(|field_type_ids| field_type_ids[i])
                .unwrap_or_default();
            let field_span = field.span();
            let typed_field = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: field,
                    namespace,
                    crate_namespace,
                    return_type_annotation: field_type_id,
                    help_text: "tuple field type does not match the expected type",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                error_recovery_expr(field_span),
                warnings,
                errors
            );
            if let IsConstant::No = typed_field.is_constant {
                is_constant = IsConstant::No;
            }
            typed_field_types.push(typed_field.return_type);
            typed_fields.push(typed_field);
        }
        let exp = TypedExpression {
            expression: TypedExpressionVariant::Tuple {
                fields: typed_fields,
            },
            return_type: crate::type_engine::insert_type(TypeInfo::Tuple(typed_field_types)),
            is_constant,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_delineated_path<'n>(
        call_path: CallPath,
        span: Span,
        args: Vec<Expression>,
        // TODO these will be needed for enum instantiation
        _type_arguments: Vec<TypeInfo>,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
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
            .find_module_relative(&call_path.prefixes)
            .ok(&mut probe_warnings, &mut probe_errors);
        let enum_module_combined_result = {
            // also, check if this is an enum _in_ another module.
            let (module_path, enum_name) =
                call_path.prefixes.split_at(call_path.prefixes.len() - 1);
            let enum_name = enum_name[0].clone();
            let namespace = namespace.find_module_relative(module_path);
            let namespace = namespace.ok(&mut warnings, &mut errors);
            namespace.map(|ns| ns.find_enum(&enum_name)).flatten()
        };

        // now we can see if this thing is a symbol (typed declaration) or reference to an
        // enum instantiation
        let this_thing: Either<TypedDeclaration, TypedExpression> =
            match (module_result, enum_module_combined_result) {
                (Some(_module), Some(_enum_res)) => {
                    errors.push(CompileError::AmbiguousPath { span });
                    return err(warnings, errors);
                }
                (Some(module), None) => match module.get_symbol(&call_path.suffix).value.cloned() {
                    Some(decl) => Either::Left(decl),
                    None => {
                        errors.push(CompileError::SymbolNotFound {
                            name: call_path.suffix.as_str().to_string(),
                            span: call_path.suffix.span().clone(),
                        });
                        return err(warnings, errors);
                    }
                },
                (None, Some(enum_decl)) => Either::Right(check!(
                    instantiate_enum(
                        enum_decl,
                        call_path.suffix,
                        args,
                        namespace,
                        crate_namespace,
                        self_type,
                        build_config,
                        dead_code_graph,
                        dependency_graph,
                        opts,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                )),
                (None, None) => {
                    errors.push(CompileError::SymbolNotFound {
                        span,
                        name: call_path.suffix.as_str().to_string(),
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

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast<'n>(
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // TODO use lib-std's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span();
        // TODO(static span): the below String address should just be address_expr
        // basically delete the bottom line and replace references to it with address_expr
        let address_str = address.span().as_str().to_string();
        let address_expr = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *address,
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::B256),
                help_text: "An address that is being ABI cast must be of type b256",
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
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
        let return_type = insert_type(TypeInfo::ContractCaller {
            abi_name: abi_name.to_owned_call_path(),
            address: address_str,
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
                TypedFunctionDeclaration::type_check(TypeCheckArguments {
                    checkee: method.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type: insert_type(TypeInfo::Contract),
                    build_config,
                    dead_code_graph,
                    mode: Mode::ImplAbiFn,
                    dependency_graph,
                    opts,
                }),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        functions_buf.append(&mut type_checked_fn_buf);
        namespace.insert_trait_implementation(
            abi_name.clone(),
            look_up_type_id(return_type),
            functions_buf,
        );
        let exp = TypedExpression {
            expression: TypedExpressionVariant::AbiCast {
                abi_name,
                address: Box::new(address_expr),
                span: span.clone(),
            },
            return_type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_array<'n>(
        contents: Vec<Expression>,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        if contents.is_empty() {
            return ok(
                TypedExpression {
                    expression: TypedExpressionVariant::Array {
                        contents: Vec::new(),
                    },
                    return_type: insert_type(TypeInfo::Array(insert_type(TypeInfo::Unknown), 0)),
                    is_constant: IsConstant::Yes,
                    span,
                },
                Vec::new(),
                Vec::new(),
            );
        };

        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let typed_contents: Vec<TypedExpression> = contents
            .into_iter()
            .map(|expr| {
                let span = expr.span();
                check!(
                    Self::type_check(TypeCheckArguments {
                        checkee: expr,
                        namespace,
                        crate_namespace,
                        return_type_annotation: insert_type(TypeInfo::Unknown),
                        help_text: Default::default(),
                        self_type,
                        build_config,
                        dead_code_graph,
                        dependency_graph,
                        mode: Mode::NonAbi,
                        opts,
                    }),
                    error_recovery_expr(span),
                    warnings,
                    errors
                )
            })
            .collect();

        let elem_type = typed_contents[0].return_type;
        for typed_elem in &typed_contents[1..] {
            match unify_with_self(
                typed_elem.return_type,
                elem_type,
                self_type,
                &typed_elem.span,
            ) {
                // In both cases, if there are warnings or errors then break here, since we don't
                // need to spam type errors for every element once we have one.
                Ok(mut ws) => {
                    let no_warnings = ws.is_empty();
                    warnings.append(&mut ws);
                    if !no_warnings {
                        break;
                    }
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                    break;
                }
            };
        }

        let array_count = typed_contents.len();
        ok(
            TypedExpression {
                expression: TypedExpressionVariant::Array {
                    contents: typed_contents,
                },
                return_type: insert_type(TypeInfo::Array(elem_type, array_count)),
                is_constant: IsConstant::No, // Maybe?
                span,
            },
            warnings,
            errors,
        )
    }

    fn type_check_array_index(
        arguments: TypeCheckArguments<'_, (Expression, Expression)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee: (prefix, index),
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            dependency_graph,
            opts,
            ..
        } = arguments;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let prefix_te = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: prefix.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            return err(warnings, errors),
            warnings,
            errors
        );

        // If the return type is a static array then create a TypedArrayIndex.
        if let TypeInfo::Array(elem_type_id, _) = look_up_type_id(prefix_te.return_type) {
            let index_te = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: index,
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::UnsignedInteger(
                        IntegerBits::SixtyFour
                    )),
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                return err(warnings, errors),
                warnings,
                errors
            );

            ok(
                TypedExpression {
                    expression: TypedExpressionVariant::ArrayIndex {
                        prefix: Box::new(prefix_te),
                        index: Box::new(index_te),
                    },
                    return_type: elem_type_id,
                    is_constant: IsConstant::No,
                    span,
                },
                warnings,
                errors,
            )
        } else {
            // Otherwise convert into a method call 'index(self, index)' via the std::ops::Index trait.
            let method_name = MethodName::FromType {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: Ident::new_with_override("index", span.clone()),
                },
                type_name: None,
                is_absolute: true,
            };
            type_check_method_application(
                method_name,
                vec![prefix, index],
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
                opts,
            )
        }
    }

    /// This function takes a [DelayedResolutionVariant] and returns either a
    /// [TypedExpressionVariant::EnumArgAccess] (given the case of enum arg
    /// access) or returns a [TypedExpressionVariant::StructFieldAccess] (given
    /// the case of struct field access). This function does several things, it
    /// 1) checks to ensure that the expression inside of the
    /// [DelayedResolutionVariant] is of the appropriate type (either an enum
    /// or a struct), 2) determines the return type of the corresponding
    /// struct field or enum arg, and 3) constructs the respective typed
    /// expression.
    fn type_check_delayed_resolution<'n>(
        variant: DelayedResolutionVariant,
        span: Span,
        namespace: &mut Namespace,
        crate_namespace: Option<&'n Namespace>,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match variant {
            DelayedResolutionVariant::TupleVariant(DelayedTupleVariantResolution {
                exp,
                elem_num,
            }) => {
                let args = TypeCheckArguments {
                    checkee: *exp,
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: "",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let parent = check!(
                    TypedExpression::type_check(args),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let tuple_elems = check!(
                    namespace.get_tuple_elems(
                        parent.return_type,
                        parent.span.as_str(),
                        &parent.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut tuple_elem_to_access = None;
                for (pos, tuple_elem) in tuple_elems.into_iter().enumerate() {
                    if pos == elem_num {
                        tuple_elem_to_access = Some(tuple_elem)
                    }
                }
                let tuple_elem_to_access = match tuple_elem_to_access {
                    None => {
                        errors.push(CompileError::MatchWrongType {
                            expected: parent.return_type,
                            span: parent.span,
                        });
                        let exp = error_recovery_expr(span);
                        return ok(exp, warnings, errors);
                    }
                    Some(tuple_elem_to_access) => tuple_elem_to_access,
                };
                let exp = TypedExpression {
                    expression: TypedExpressionVariant::TupleElemAccess {
                        resolved_type_of_parent: parent.return_type,
                        prefix: Box::new(parent),
                        elem_to_access_num: elem_num,
                        elem_to_access_span: span.clone(),
                    },
                    return_type: tuple_elem_to_access,
                    is_constant: IsConstant::No,
                    span,
                };
                ok(exp, warnings, errors)
            }
            DelayedResolutionVariant::EnumVariant(DelayedEnumVariantResolution {
                exp,
                call_path,
                arg_num,
            }) => {
                let args = TypeCheckArguments {
                    checkee: *exp,
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: "",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let parent = check!(
                    TypedExpression::type_check(args),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let enum_name = call_path.prefixes.first().unwrap().clone();
                let variant_name = call_path.suffix.clone();
                let enum_module_combined_result = {
                    let (module_path, enum_name) =
                        call_path.prefixes.split_at(call_path.prefixes.len() - 1);
                    let enum_name = enum_name[0].clone();
                    let namespace = namespace.find_module_relative(module_path);
                    let namespace = namespace.ok(&mut warnings, &mut errors);
                    namespace.map(|ns| ns.find_enum(&enum_name)).flatten()
                };
                let mut return_type = None;
                let mut owned_enum_variant = None;
                match enum_module_combined_result {
                    None => todo!(),
                    Some(enum_decl) => {
                        if enum_name != enum_decl.name {
                            errors.push(CompileError::MatchWrongType {
                                expected: parent.return_type,
                                span: enum_name.span().clone(),
                            });
                            let exp = error_recovery_expr(span);
                            return ok(exp, warnings, errors);
                        }
                        for (pos, variant) in enum_decl.variants.into_iter().enumerate() {
                            match (pos == arg_num, variant.name == variant_name) {
                                (true, true) => {
                                    return_type = Some(variant.r#type);
                                    owned_enum_variant = Some(variant);
                                }
                                (true, false) => {
                                    errors.push(CompileError::MatchWrongType {
                                        expected: parent.return_type,
                                        span: variant_name.span().clone(),
                                    });
                                    let exp = error_recovery_expr(span);
                                    return ok(exp, warnings, errors);
                                }
                                _ => (),
                            }
                        }
                    }
                }
                let (return_type, _owned_enum_variant) = match (return_type, owned_enum_variant) {
                    (Some(return_type), Some(owned_enum_variant)) => {
                        (return_type, owned_enum_variant)
                    }
                    _ => {
                        errors.push(CompileError::MatchWrongType {
                            expected: parent.return_type,
                            span: enum_name.span().clone(),
                        });
                        let exp = error_recovery_expr(span);
                        return ok(exp, warnings, errors);
                    }
                };

                let exp = TypedExpression {
                    expression: TypedExpressionVariant::EnumArgAccess {
                        resolved_type_of_parent: parent.return_type,
                        prefix: Box::new(parent),
                        //variant_to_access: owned_enum_variant.to_owned(),
                        arg_num_to_access: arg_num,
                    },
                    return_type,
                    is_constant: IsConstant::No,
                    span,
                };
                ok(exp, warnings, errors)
            }
            DelayedResolutionVariant::StructField(DelayedStructFieldResolution {
                exp,
                struct_name,
                field,
            }) => {
                let args = TypeCheckArguments {
                    checkee: *exp,
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: "",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
                    mode: Mode::NonAbi,
                    opts,
                };
                let parent = check!(
                    TypedExpression::type_check(args),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let (struct_fields, other_struct_name) = check!(
                    namespace.get_struct_type_fields(
                        parent.return_type,
                        parent.span.as_str(),
                        &parent.span
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                if struct_name.as_str() != other_struct_name {
                    errors.push(CompileError::MatchWrongType {
                        expected: parent.return_type,
                        span: struct_name.span().clone(),
                    });
                    let exp = error_recovery_expr(span);
                    return ok(exp, warnings, errors);
                }
                let mut field_to_access = None;
                for struct_field in struct_fields.iter() {
                    if struct_field.name == *field.as_str() {
                        field_to_access = Some(struct_field.clone())
                    }
                }
                let field_to_access = match field_to_access {
                    None => {
                        errors.push(CompileError::MatchWrongType {
                            expected: parent.return_type,
                            span: struct_name.span().clone(),
                        });
                        let exp = error_recovery_expr(span);
                        return ok(exp, warnings, errors);
                    }
                    Some(field_to_access) => field_to_access,
                };
                let exp = TypedExpression {
                    expression: TypedExpressionVariant::StructFieldAccess {
                        resolved_type_of_parent: parent.return_type,
                        prefix: Box::new(parent),
                        field_to_access: field_to_access.clone(),
                        field_to_access_span: field.span().clone(),
                    },
                    return_type: field_to_access.r#type,
                    is_constant: IsConstant::No,
                    span,
                };
                ok(exp, warnings, errors)
            }
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} ({})",
            self.expression.pretty_print(),
            look_up_type_id(self.return_type).friendly_type_str()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_type_check(expr: Expression, type_annotation: TypeId) -> CompileResult<TypedExpression> {
        let mut namespace: Namespace = Default::default();
        let self_type = insert_type(TypeInfo::Unknown);
        let build_config = BuildConfig {
            file_name: Arc::new("test.sw".into()),
            dir_of_code: Arc::new("".into()),
            manifest_path: Arc::new("".into()),
            print_intermediate_asm: false,
            print_finalized_asm: false,
        };
        let mut dead_code_graph: ControlFlowGraph = Default::default();
        let mut dependency_graph = HashMap::new();

        TypedExpression::type_check(TypeCheckArguments {
            checkee: expr,
            namespace: &mut namespace,
            crate_namespace: None,
            return_type_annotation: type_annotation,
            help_text: Default::default(),
            self_type,
            build_config: &build_config,
            dead_code_graph: &mut dead_code_graph,
            dependency_graph: &mut dependency_graph,
            mode: Mode::NonAbi,
            opts: Default::default(),
        })
    }

    fn do_type_check_for_boolx2(expr: Expression) -> CompileResult<TypedExpression> {
        do_type_check(
            expr,
            insert_type(TypeInfo::Array(insert_type(TypeInfo::Boolean), 2)),
        )
    }

    #[test]
    fn test_array_type_check_non_homogeneous_0() {
        let empty_span = Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        };

        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: empty_span.clone(),
                },
                Expression::Literal {
                    value: Literal::U64(0),
                    span: empty_span.clone(),
                },
            ],
            span: empty_span,
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.friendly_type_str() == "bool"
                                && received.friendly_type_str() == "u64"));
    }

    #[test]
    fn test_array_type_check_non_homogeneous_1() {
        let empty_span = Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        };

        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::U64(0),
                    span: empty_span.clone(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: empty_span.clone(),
                },
            ],
            span: empty_span,
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 2);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.friendly_type_str() == "u64"
                                && received.friendly_type_str() == "bool"));
        assert!(matches!(&comp_res.errors[1],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.friendly_type_str() == "[bool; 2]"
                                && received.friendly_type_str() == "[u64; 2]"));
    }

    #[test]
    fn test_array_type_check_bad_count() {
        let empty_span = Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        };

        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: empty_span.clone(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: empty_span.clone(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: empty_span.clone(),
                },
            ],
            span: empty_span,
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.friendly_type_str() == "[bool; 2]"
                                && received.friendly_type_str() == "[bool; 3]"));
    }

    #[test]
    fn test_array_type_check_empty() {
        let empty_span = Span {
            span: pest::Span::new(" ".into(), 0, 0).unwrap(),
            path: None,
        };

        let expr = Expression::Array {
            contents: Vec::new(),
            span: empty_span,
        };

        let comp_res = do_type_check(
            expr,
            insert_type(TypeInfo::Array(insert_type(TypeInfo::Boolean), 0)),
        );
        assert!(comp_res.warnings.is_empty() && comp_res.errors.is_empty());
    }
}
