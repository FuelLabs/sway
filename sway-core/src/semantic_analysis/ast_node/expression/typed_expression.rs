use super::*;

use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    semantic_analysis::ast_node::*,
    type_engine::{insert_type, IntegerBits},
};

mod method_application;
use crate::type_engine::TypeId;
use method_application::type_check_method_application;

#[derive(Clone, Debug)]
pub struct TypedExpression {
    pub(crate) expression: TypedExpressionVariant,
    pub(crate) return_type: TypeId,
    /// whether or not this expression is constantly evaluable (if the result is known at compile
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
    pub(crate) fn deterministically_aborts(&self) -> bool {
        use TypedExpressionVariant::*;
        match &self.expression {
            FunctionApplication {
                function_body,
                arguments,
                ..
            } => {
                function_body.deterministically_aborts()
                    || arguments.iter().any(|(_, x)| x.deterministically_aborts())
            }
            Tuple { fields, .. } => fields.iter().any(|x| x.deterministically_aborts()),
            Array { contents, .. } => contents.iter().any(|x| x.deterministically_aborts()),
            CodeBlock(contents) => contents.deterministically_aborts(),
            LazyOperator { lhs, .. } => lhs.deterministically_aborts(),
            StructExpression { fields, .. } => {
                fields.iter().any(|x| x.value.deterministically_aborts())
            }
            EnumInstantiation { contents, .. } => contents
                .as_ref()
                .map(|x| x.deterministically_aborts())
                .unwrap_or(false),
            AbiCast { address, .. } => address.deterministically_aborts(),
            SizeOf {
                variant: SizeOfVariant::Val(v),
            } => v.deterministically_aborts(),
            StructFieldAccess { .. }
            | Literal(_)
            | StorageAccess { .. }
            | SizeOf { .. }
            | VariableExpression { .. }
            | FunctionParameter
            | TupleElemAccess { .. } => false,
            ArrayIndex { prefix, index } => {
                prefix.deterministically_aborts() || index.deterministically_aborts()
            }
            AsmExpression {
                registers, body, ..
            } => {
                // when asm expression parsing is handled earlier, this will be cleaner. For now,
                // we rely on string comparison...
                // jumps are not allowed in asm blocks, so we know this block deterministically
                // aborts if these opcodes are present
                let body_deterministically_aborts = body
                    .iter()
                    .any(|x| ["rvrt", "ret"].contains(&x.op_name.as_str().to_lowercase().as_str()));
                registers.iter().any(|x| {
                    x.initializer
                        .as_ref()
                        .map(|x| x.deterministically_aborts())
                        .unwrap_or(false)
                }) || body_deterministically_aborts
            }
            IfExp {
                condition,
                then,
                r#else,
                ..
            } => {
                condition.deterministically_aborts()
                    || (then.deterministically_aborts()
                        && r#else
                            .as_ref()
                            .map(|x| x.deterministically_aborts())
                            .unwrap_or(false))
            }
            IfLet {
                expr, then, r#else, ..
            } => {
                expr.deterministically_aborts()
                    || (then.deterministically_aborts()
                        && r#else
                            .as_ref()
                            .map(|x| x.deterministically_aborts())
                            .unwrap_or(false))
            }
        }
    }
    /// recurse into `self` and get any return statements -- used to validate that all returns
    /// do indeed return the correct type
    /// This does _not_ extract implicit return statements as those are not control flow! This is
    /// _only_ for explicit returns.
    pub(crate) fn gather_return_statements(&self) -> Vec<&TypedReturnStatement> {
        match &self.expression {
            TypedExpressionVariant::IfExp {
                condition,
                then,
                r#else,
            } => {
                let mut buf = condition.gather_return_statements();
                buf.append(&mut then.gather_return_statements());
                if let Some(ref r#else) = r#else {
                    buf.append(&mut r#else.gather_return_statements());
                }
                buf
            }
            TypedExpressionVariant::IfLet {
                expr, then, r#else, ..
            } => {
                let mut buf = expr.gather_return_statements();
                for node in &then.contents {
                    buf.append(&mut node.gather_return_statements())
                }
                if let Some(ref r#else) = r#else {
                    buf.append(&mut r#else.gather_return_statements())
                }
                buf
            }
            TypedExpressionVariant::CodeBlock(TypedCodeBlock { contents, .. }) => {
                let mut buf = vec![];
                for node in contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            // if it is impossible for an expression to contain a return _statement_ (not an
            // implicit return!), put it in the pattern below.
            TypedExpressionVariant::LazyOperator { .. }
            | TypedExpressionVariant::Literal(_)
            | TypedExpressionVariant::Tuple { .. }
            | TypedExpressionVariant::Array { .. }
            | TypedExpressionVariant::ArrayIndex { .. }
            | TypedExpressionVariant::FunctionParameter { .. }
            | TypedExpressionVariant::AsmExpression { .. }
            | TypedExpressionVariant::StructFieldAccess { .. }
            | TypedExpressionVariant::TupleElemAccess { .. }
            | TypedExpressionVariant::EnumInstantiation { .. }
            | TypedExpressionVariant::AbiCast { .. }
            | TypedExpressionVariant::SizeOf { .. }
            | TypedExpressionVariant::StructExpression { .. }
            | TypedExpressionVariant::VariableExpression { .. }
            | TypedExpressionVariant::StorageAccess { .. }
            | TypedExpressionVariant::FunctionApplication { .. } => vec![],
        }
    }
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
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
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
                    help_text,
                    self_type,
                    build_config,
                    dead_code_graph,
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
                    mode: Mode::NonAbi,
                    help_text: Default::default(),
                    opts,
                },
                span,
            ),
            Expression::MatchExp {
                if_exp,
                span,
                cases_covered,
            } => Self::type_check_match_expression(
                TypeCheckArguments {
                    checkee: (*if_exp, cases_covered),
                    return_type_annotation: type_annotation,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
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
                opts,
            ),
            Expression::MethodApplication {
                method_name,
                contract_call_params,
                arguments,
                span,
            } => type_check_method_application(
                method_name,
                contract_call_params,
                arguments,
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
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
                opts,
            ),
            Expression::TupleIndex {
                prefix,
                index,
                index_span,
                span,
            } => Self::type_check_tuple_index(
                *prefix,
                index,
                index_span,
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
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
                    opts,
                )
            }
            Expression::IfLet {
                scrutinee,
                expr,
                then,
                r#else,
                span,
            } => Self::type_check_if_let_expression(
                TypeCheckArguments {
                    checkee: (scrutinee, expr, then, r#else),
                    return_type_annotation: type_annotation,
                    mode: Mode::NonAbi,
                    help_text: Default::default(),
                    opts,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                },
                span,
            ),

            Expression::SizeOfVal { exp, span } => Self::type_check_size_of_val(
                TypeCheckArguments {
                    checkee: *exp,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                },
                span,
            ),
            Expression::SizeOfType {
                type_name,
                type_span,
                span,
            } => Self::type_check_size_of_type(
                TypeCheckArguments {
                    checkee: (type_name, type_span),
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    opts,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    mode: Default::default(),
                    help_text: Default::default(),
                },
                span,
            ),
            /*
            a => {
                let errors = vec![CompileError::Unimplemented(
                    "Unimplemented type checking for expression",
                    a.span(),
                )];

                let exp = error_recovery_expr(a.span());
                ok(exp, vec![], errors)
            }
            */
        };
        let mut typed_expression = match res.value {
            Some(r) => r,
            None => return res,
        };
        let mut warnings = res.warnings;
        let mut errors = res.errors;
        // if one of the expressions deterministically aborts, we don't want to type check it.
        if !typed_expression.deterministically_aborts() {
            // if the return type cannot be cast into the annotation type then it is a type error
            match unify_with_self(
                typed_expression.return_type,
                type_annotation,
                self_type,
                &expr_span,
                help_text,
            ) {
                Ok(mut ws) => {
                    warnings.append(&mut ws);
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            }
        };
        // The annotation may result in a cast, which is handled in the type engine.

        typed_expression.return_type = namespace
            .resolve_type_with_self(look_up_type_id(typed_expression.return_type), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType {
                    span: expr_span.clone(),
                });
                insert_type(TypeInfo::ErrorRecovery)
            });

        // Literals of type Numeric can now be resolved if typed_expression.return_type is
        // an UnsignedInteger or a Numeric
        if let TypedExpressionVariant::Literal(lit) = typed_expression.clone().expression {
            if let Literal::Numeric(_) = lit {
                match look_up_type_id(typed_expression.return_type) {
                    TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => {
                        typed_expression = check!(
                            Self::resolve_numeric_literal(
                                lit,
                                expr_span,
                                typed_expression.return_type
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    _ => {}
                }
            }
        }

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
            Literal::Numeric(_) => TypeInfo::Numeric,
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

    pub(crate) fn type_check_variable_expression(
        name: Ident,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
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
        let typed_function_decl =
            if let TypedDeclaration::FunctionDeclaration(decl) = function_declaration {
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
        instantiate_function_application(
            typed_function_decl,
            name,
            arguments,
            namespace,
            crate_namespace,
            self_type,
            build_config,
            dead_code_graph,
            opts,
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

    fn type_check_code_block(
        contents: CodeBlock,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        type_annotation: TypeId,
        help_text: &'static str,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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

        match unify_with_self(
            block_return_type,
            type_annotation,
            self_type,
            &span,
            help_text,
        ) {
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
    fn type_check_if_let_expression(
        arguments: TypeCheckArguments<
            '_,
            (
                Scrutinee,
                Box<Expression>,
                CodeBlock,
                Option<Box<Expression>>,
            ),
        >,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee: (scrutinee, expr, then, r#else),
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            self_type,
            build_config,
            dead_code_graph,

            opts,
            ..
        } = arguments;
        let mut warnings = vec![];
        let mut errors = vec![];
        let (enum_type, variant) = check!(
            check_scrutinee_type(&scrutinee, namespace, self_type),
            return err(warnings, errors),
            warnings,
            errors
        );
        let variable_to_assign = check!(
            scrutinee.enum_variable_to_assign(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let expr = Box::new(check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *expr.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: enum_type,
                help_text: "The return value of this expression must match the type of the pattern provided.",
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(expr.span()),
            warnings,
            errors
        ));
        // put the variable and type of the enum variants inner type into the namespace for the
        // "then" branch but not the else branch
        let then_branch_scope = create_new_scope(namespace);
        // calculate the return type of the variable by checking the enum variant's return type

        then_branch_scope.insert(
            variable_to_assign.clone(),
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                name: variable_to_assign.clone(),
                type_ascription: variant.r#type,
                is_mutable: VariableMutability::Immutable, // TODO allow mut variables in patterns? grammar can't handle it right now
                const_decl_origin: false,
                body: TypedExpression {
                    expression: TypedExpressionVariant::Tuple { fields: vec![] },
                    return_type: variant.r#type,
                    is_constant: IsConstant::No,
                    span: span.clone(),
                },
            }),
        );

        let then_branch_span = then.span().clone();

        // type check the then branch
        let (then, then_branch_code_block_return_type) = check!(
            TypedCodeBlock::type_check(TypeCheckArguments {
                checkee: then,
                namespace: then_branch_scope,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: "Because the return value of this expression is used, all branches of `if let` expression must return this type",
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            (
                TypedCodeBlock {
                    contents: Default::default(),
                    whole_block_span: then_branch_span.clone()
                },
                insert_type(TypeInfo::ErrorRecovery)
            ),
            warnings,
            errors
        );

        // if the branch aborts, then its return type doesn't matter.
        if !then.deterministically_aborts() {
            // if this does not deterministically_abort, check the block return type
            let ty_to_check = if r#else.is_some() {
                type_annotation
            } else {
                insert_type(TypeInfo::Tuple(vec![]))
            };
            match unify_with_self(
                then_branch_code_block_return_type,
                ty_to_check,
                self_type,
                then.span(),
                "`then` branch must return expected type.",
            ) {
                Ok(mut ws) => {
                    warnings.append(&mut ws);
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            };
        }

        let r#else = match r#else {
            Some(expr) => {
                let expr_span = expr.span();
                let r#else= check!(
                    TypedExpression::type_check(TypeCheckArguments {
                        checkee: *expr,
                        namespace,
                        crate_namespace,
                        return_type_annotation: insert_type(TypeInfo::Unknown),
                        help_text:
                            "The two branches of an if let expression must return the same type.",
                        self_type,
                        build_config,
                        dead_code_graph,
                        mode: Mode::NonAbi,
                        opts,
                    }),
                    error_recovery_expr(expr_span),
                    warnings,
                    errors
                );

            if !r#else.deterministically_aborts() {
                // if this does not deterministically_abort, check the block return type
                match unify_with_self(
                    r#else.return_type,then_branch_code_block_return_type,
                    self_type,
                    &r#else.span,
                    "`else` branch must return expected type.",
                ) {
                    Ok(mut ws) => {
                        warnings.append(&mut ws);
                    }
                    Err(e) => {
                        errors.push(CompileError::TypeError(e));
                    }
                };
            }
                Some(Box::new(r#else))
            }
            None => match unify_with_self(
                then_branch_code_block_return_type,
                insert_type(TypeInfo::Tuple(vec![])),
                self_type,
                &then_branch_span,
                "Because this `if let` doesn't have an else branch, it cannot implicitly return any value."
            ) {
                Ok(_) => None,
                Err(e) => {
                    errors.push(e.into());
                    return err(warnings, errors);
                }
            },
        };

        let exp = TypedExpression {
            expression: TypedExpressionVariant::IfLet {
                expr,
                variable_to_assign: variable_to_assign.clone(),
                enum_type,
                variant,
                then,
                r#else,
            },
            is_constant: IsConstant::No, // TODO
            return_type: then_branch_code_block_return_type,
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
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(then.span()),
            warnings,
            errors
        ));
        // if the branch aborts, then its return type doesn't matter.
        let then_deterministically_aborts = then.deterministically_aborts();
        if !then_deterministically_aborts {
            // if this does not deterministically_abort, check the block return type
            let ty_to_check = if r#else.is_some() {
                type_annotation
            } else {
                insert_type(TypeInfo::Tuple(vec![]))
            };
            match unify_with_self(
                then.return_type,
                ty_to_check,
                self_type,
                &then.span,
                "`then` branch must return expected type.",
            ) {
                Ok(mut ws) => {
                    warnings.append(&mut ws);
                }
                Err(e) => {
                    errors.push(CompileError::TypeError(e));
                }
            };
        }
        let mut else_deterministically_aborts = false;
        let r#else = r#else.map(|expr| {
            let r#else = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: *expr.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                }),
                error_recovery_expr(expr.span()),
                warnings,
                errors
            );
            else_deterministically_aborts = r#else.deterministically_aborts();
            if !else_deterministically_aborts {
                // if this does not deterministically_abort, check the block return type
                match unify_with_self(
                    r#else.return_type,
                    then.return_type,
                    self_type,
                    &r#else.span,
                    "`else` branch must return expected type.",
                ) {
                    Ok(mut ws) => {
                        warnings.append(&mut ws);
                    }
                    Err(e) => {
                        errors.push(CompileError::TypeError(e));
                    }
                };
            }
            Box::new(r#else)
        });

        let r#else_ret_ty = r#else
            .as_ref()
            .map(|x| x.return_type)
            .unwrap_or_else(|| insert_type(TypeInfo::Tuple(Vec::new())));
        // if there is a type annotation, then the else branch must exist
        if !else_deterministically_aborts && !then_deterministically_aborts {
            match unify_with_self(
                then.return_type,
                r#else_ret_ty,
                self_type,
                &span,
                "The two branches of an if expression must return the same type.",
            ) {
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

    #[allow(clippy::type_complexity)]
    fn type_check_match_expression(
        arguments: TypeCheckArguments<'_, (Expression, Vec<MatchCondition>)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (if_exp, cases_covered),
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
        let args = TypeCheckArguments {
            checkee: if_exp.clone(),
            namespace,
            crate_namespace,
            return_type_annotation: type_annotation,
            help_text: Default::default(),
            self_type,
            build_config,
            dead_code_graph,
            mode: Mode::NonAbi,
            opts,
        };
        let typed_if_exp = check!(
            TypedExpression::type_check(args),
            error_recovery_expr(if_exp.span()),
            warnings,
            errors
        );
        let (witness_report, arms_reachability) = check!(
            check_match_expression_usefulness(cases_covered, span.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        for (arm, reachable) in arms_reachability.into_iter() {
            if !reachable {
                warnings.push(CompileWarning {
                    span: arm.span(),
                    warning_content: Warning::MatchExpressionUnreachableArm,
                });
            }
        }
        if witness_report.has_witnesses() {
            errors.push(CompileError::MatchExpressionNonExhaustive {
                missing_patterns: format!("{}", witness_report),
                span,
            });
            return err(warnings, errors);
        }
        ok(typed_if_exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        asm: AsmExpression,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
    fn type_check_struct_expression(
        span: Span,
        call_path: CallPath,
        fields: Vec<StructExpressionField>,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let module_result = namespace
            .find_module_relative(&call_path.prefixes)
            .ok(&mut warnings, &mut errors);
        let mut typed_fields_buf = vec![];
        let definition = match module_result {
            Some(module) => match module.clone().get_symbol(&call_path.suffix).value {
                Some(TypedDeclaration::StructDeclaration(st)) => st,
                Some(_) => {
                    errors.push(CompileError::DeclaredNonStructAsStruct {
                        name: call_path.suffix.clone(),
                        span,
                    });
                    return err(warnings, errors);
                }
                None => {
                    errors.push(CompileError::StructNotFound {
                        name: call_path.suffix.clone(),
                        span,
                    });
                    return err(warnings, errors);
                }
            },
            None => {
                errors.push(CompileError::StructNotFound {
                    name: call_path.suffix,
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
        let struct_type_id = insert_type(TypeInfo::Struct {
            name: definition.name.clone(),
            fields: definition.fields.clone(),
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
    fn type_check_subfield_expression(
        prefix: Box<Expression>,
        span: Span,
        field_to_access: Ident,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
            .find(|TypedStructField { name, .. }| name.as_str() == field_to_access.as_str())
        {
            field
        } else {
            errors.push(CompileError::FieldNotFound {
                span: field_to_access.span().clone(),
                available_fields: fields
                    .iter()
                    .map(|TypedStructField { name, .. }| name.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                field_name: field_to_access.clone(),
                struct_name: struct_name.to_string(),
            });
            return err(warnings, errors);
        };

        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructFieldAccess {
                resolved_type_of_parent: parent.return_type,
                prefix: Box::new(parent),
                field_to_access: field.clone(),
            },
            return_type: field.r#type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_tuple(
        fields: Vec<Expression>,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        type_annotation: TypeId,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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

    fn type_check_tuple_index(
        prefix: Expression,
        index: usize,
        index_span: Span,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let parent = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: prefix,
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut tuple_elem_to_access = None;
        let tuple_elems = check!(
            namespace.get_tuple_elems(parent.return_type, parent.span.as_str(), &parent.span),
            return err(warnings, errors),
            warnings,
            errors
        );
        for (pos, elem) in tuple_elems.iter().enumerate() {
            if pos == index {
                tuple_elem_to_access = Some(*elem);
            }
        }
        let tuple_elem_to_access = match tuple_elem_to_access {
            Some(tuple_elem_to_access) => tuple_elem_to_access,
            None => {
                errors.push(CompileError::TupleOutOfBounds {
                    index,
                    count: tuple_elems.len(),
                    span: index_span,
                });
                return err(warnings, errors);
            }
        };
        let exp = TypedExpression {
            expression: TypedExpressionVariant::TupleElemAccess {
                resolved_type_of_parent: parent.return_type,
                prefix: Box::new(parent),
                elem_to_access_num: index,
                elem_to_access_span: index_span,
            },
            return_type: tuple_elem_to_access,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_delineated_path(
        call_path: CallPath,
        span: Span,
        args: Vec<Expression>,
        type_arguments: Vec<(TypeInfo, Span)>,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // The first step is to determine if the call path refers to a module, enum, or function.
        // We could rely on the capitalization convention, where modules are lowercase
        // and enums are uppercase, but this is not robust in the long term.
        // Instead, we try to resolve all paths.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
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
            namespace.and_then(|ns| ns.find_enum(&enum_name))
        };

        // now we can see if this thing is a symbol (typed declaration) or reference to an
        // enum instantiation, and if it is not either of those things, then it might be a
        // function application
        let exp: TypedExpression = match (module_result, enum_module_combined_result) {
            (Some(_module), Some(_enum_res)) => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            (Some(module), None) => match module.get_symbol(&call_path.suffix).value {
                Some(decl) => match decl {
                    TypedDeclaration::EnumDeclaration(enum_decl) => {
                        check!(
                            instantiate_enum(
                                enum_decl,
                                call_path.suffix,
                                args,
                                type_arguments,
                                namespace,
                                crate_namespace,
                                self_type,
                                build_config,
                                dead_code_graph,
                                opts,
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    TypedDeclaration::FunctionDeclaration(func_decl) => check!(
                        instantiate_function_application(
                            func_decl,
                            call_path,
                            args,
                            namespace,
                            crate_namespace,
                            self_type,
                            build_config,
                            dead_code_graph,
                            opts,
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                    a => {
                        errors.push(CompileError::NotAnEnum {
                            name: call_path.friendly_name(),
                            span,
                            actually: a.friendly_name().to_string(),
                        });
                        return err(warnings, errors);
                    }
                },
                None => {
                    errors.push(CompileError::SymbolNotFound {
                        name: call_path.suffix.as_str().to_string(),
                        span: call_path.suffix.span().clone(),
                    });
                    return err(warnings, errors);
                }
            },
            (None, Some(enum_decl)) => check!(
                instantiate_enum(
                    enum_decl,
                    call_path.suffix,
                    args,
                    type_arguments,
                    namespace,
                    crate_namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    opts,
                ),
                return err(warnings, errors),
                warnings,
                errors
            ),
            (None, None) => {
                errors.push(CompileError::SymbolNotFound {
                    name: call_path.suffix.as_str().to_string(),
                    span: call_path.suffix.span().clone(),
                });
                return err(warnings, errors);
            }
        };

        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
    fn type_check_array(
        contents: Vec<Expression>,
        span: Span,
        namespace: crate::semantic_analysis::NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
                "",
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
                    is_absolute: true,
                },
                type_name: None,
            };
            type_check_method_application(
                method_name,
                vec![],
                vec![prefix, index],
                span,
                namespace,
                crate_namespace,
                self_type,
                build_config,
                dead_code_graph,
                opts,
            )
        }
    }

    /// This function takes a [DelayedResolutionVariant] and returns either a
    /// [TypedExpressionVariant::IfLet] (given the case of enum arg
    /// access) or returns a [TypedExpressionVariant::StructFieldAccess] (given
    /// the case of struct field access). This function does several things, it
    /// 1) checks to ensure that the expression inside of the
    /// [DelayedResolutionVariant] is of the appropriate type (either an enum
    /// or a struct), 2) determines the return type of the corresponding
    /// struct field or enum arg, and 3) constructs the respective typed
    /// expression.
    fn type_check_delayed_resolution(
        variant: DelayedResolutionVariant,
        span: Span,
        namespace: NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
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
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
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
                call_path,
                ..
            }) => {
                errors.push(CompileError::Unimplemented(
                    "Pattern matching of enum types in this position has not yet been implemented",
                    call_path.span(),
                ));
                err(warnings, errors)
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
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
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
                if struct_name.as_str() != other_struct_name.as_str() {
                    errors.push(CompileError::MatchWrongType {
                        expected: parent.return_type,
                        span: struct_name.span().clone(),
                    });
                    let exp = error_recovery_expr(span);
                    return ok(exp, warnings, errors);
                }
                let mut field_to_access = None;
                for struct_field in struct_fields.iter() {
                    if struct_field.name.as_str() == field.as_str() {
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
                    },
                    return_type: field_to_access.r#type,
                    is_constant: IsConstant::No,
                    span,
                };
                ok(exp, warnings, errors)
            }
        }
    }

    fn type_check_size_of_val(
        arguments: TypeCheckArguments<'_, Expression>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let exp = check!(
            TypedExpression::type_check(arguments),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = TypedExpression {
            expression: TypedExpressionVariant::SizeOf {
                variant: SizeOfVariant::Val(Box::new(exp)),
            },
            return_type: crate::type_engine::insert_type(TypeInfo::UnsignedInteger(
                IntegerBits::SixtyFour,
            )),
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_size_of_type(
        arguments: TypeCheckArguments<'_, (TypeInfo, Span)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (type_name, type_span),
            self_type,
            namespace,
            ..
        } = arguments;
        let type_id = namespace
            .resolve_type_with_self(type_name, self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType { span: type_span });
                insert_type(TypeInfo::ErrorRecovery)
            });
        let exp = TypedExpression {
            expression: TypedExpressionVariant::SizeOf {
                variant: SizeOfVariant::Type(type_id),
            },
            return_type: crate::type_engine::insert_type(TypeInfo::UnsignedInteger(
                IntegerBits::SixtyFour,
            )),
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn resolve_numeric_literal(
        lit: Literal,
        span: Span,
        new_type: TypeId,
    ) -> CompileResult<TypedExpression> {
        let mut errors = vec![];
        let pest_span = span.clone().span;
        let path = span.clone().path;

        // Parse and resolve a Numeric(span) based on new_type.
        let (val, new_integer_type) = match lit {
            Literal::Numeric(num) => match look_up_type_id(new_type) {
                TypeInfo::UnsignedInteger(n) => match n {
                    IntegerBits::Eight => (
                        num.to_string().parse().map(Literal::U8).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Eight),
                                pest_span,
                                path,
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::Sixteen => (
                        num.to_string().parse().map(Literal::U16).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                                pest_span,
                                path,
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::ThirtyTwo => (
                        num.to_string().parse().map(Literal::U32).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                                pest_span,
                                path,
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::SixtyFour => (
                        num.to_string().parse().map(Literal::U64).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                pest_span,
                                path,
                            )
                        }),
                        new_type,
                    ),
                },
                TypeInfo::Numeric => (
                    num.to_string().parse().map(Literal::U64).map_err(|e| {
                        Literal::handle_parse_int_error(
                            e,
                            TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                            pest_span,
                            path,
                        )
                    }),
                    insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
                ),
                _ => unreachable!("Unexpected type for integer literals"),
            },
            _ => unreachable!("Unexpected non-integer literals"),
        };

        match val {
            Ok(v) => {
                let exp = TypedExpression {
                    expression: TypedExpressionVariant::Literal(v),
                    return_type: new_integer_type,
                    is_constant: IsConstant::Yes,
                    span,
                };
                ok(exp, vec![], vec![])
            }
            Err(e) => {
                errors.push(e);
                let exp = error_recovery_expr(span);
                ok(exp, vec![], errors)
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

fn check_scrutinee_type(
    scrutinee: &Scrutinee,
    namespace: NamespaceRef,
    self_type: TypeId,
) -> CompileResult<(TypeId, TypedEnumVariant)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (ty, enum_variant) = match scrutinee {
        Scrutinee::EnumScrutinee { ref call_path, .. } => check!(
            check_enum_scrutinee_type(call_path, namespace, self_type),
            return err(warnings, errors),
            warnings,
            errors
        ),
        _ => {
            errors.push(CompileError::Unimplemented(
                "Destructuring this type is not yet implemented.",
                scrutinee.span(),
            ));
            return err(warnings, errors);
        }
    };

    ok((ty.as_type(), enum_variant), warnings, errors)
}

fn check_enum_scrutinee_type(
    call_path: &CallPath,
    namespace: NamespaceRef,
    self_type: TypeId,
) -> CompileResult<(TypedEnumDeclaration, TypedEnumVariant)> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let enum_variant = call_path.suffix.clone();
    let call_path = call_path.rshift();
    let decl: TypedDeclaration = check!(
        namespace.get_call_path(&call_path),
        return err(warnings, errors),
        warnings,
        errors
    );
    let enum_decl = match decl {
        TypedDeclaration::EnumDeclaration(decl) => decl,
        _ => {
            errors.push(CompileError::IfLetNonEnum {
                span: call_path.span(),
            });
            return err(warnings, errors);
        }
    };
    let enum_decl = if !enum_decl.type_parameters.is_empty() {
        // infallible because it is an invariant that monomorphization can only fail if
        // type arguments are passed in.
        // we are passing in an empty vector of type arguments here, so the result
        // is infallible.
        infallible(enum_decl.monomorphize(vec![], self_type))
    } else {
        enum_decl
    };
    // ensure the variant is in the decl
    let matching_variant = enum_decl
        .variants()
        .iter()
        .find(|TypedEnumVariant { name, .. }| *name == enum_variant)
        .cloned();
    match matching_variant {
        Some(variant) => ok((enum_decl, variant), warnings, errors),
        None => {
            errors.push(CompileError::UnknownEnumVariant {
                variant_name: enum_variant,
                enum_name: enum_decl.name,
                span: call_path.span(),
            });
            err(warnings, errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    fn do_type_check(expr: Expression, type_annotation: TypeId) -> CompileResult<TypedExpression> {
        let namespace = create_module();
        let self_type = insert_type(TypeInfo::Unknown);
        let build_config = BuildConfig {
            file_name: Arc::new("test.sw".into()),
            dir_of_code: Arc::new("".into()),
            manifest_path: Arc::new("".into()),
            use_ir: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
            generated_names: Arc::new(Mutex::new(vec![])),
        };
        let mut dead_code_graph: ControlFlowGraph = Default::default();

        TypedExpression::type_check(TypeCheckArguments {
            checkee: expr,
            namespace,
            crate_namespace: namespace,
            return_type_annotation: type_annotation,
            help_text: Default::default(),
            self_type,
            build_config: &build_config,
            dead_code_graph: &mut dead_code_graph,
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
