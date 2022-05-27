mod enum_instantiation;
mod function_application;
mod if_expression;
mod lazy_operator;
mod method_application;
mod struct_field_access;
mod tuple_index_access;
mod unsafe_downcast;

use std::collections::{HashMap, VecDeque};

pub(crate) use self::{
    enum_instantiation::*, function_application::*, if_expression::*, lazy_operator::*,
    method_application::*, struct_field_access::*, tuple_index_access::*, unsafe_downcast::*,
};

use super::match_expression::{check_match_expression_usefulness, TypedMatchExpression};

use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    semantic_analysis::ast_node::*,
    type_engine::TypeId,
    type_engine::{insert_type, AbiName, IntegerBits},
};

use ast_node::declaration::CreateTypeId;

#[derive(Clone, Debug, Eq)]
pub struct TypedExpression {
    pub(crate) expression: TypedExpressionVariant,
    pub(crate) return_type: TypeId,
    /// whether or not this expression is constantly evaluable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
    pub(crate) span: Span,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for TypedExpression {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
            && look_up_type_id(self.return_type) == look_up_type_id(other.return_type)
            && self.is_constant == other.is_constant
    }
}

impl CopyTypes for TypedExpression {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type.update_type(type_mapping, &self.span);
        self.expression.copy_types(type_mapping);
    }
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
    pub(crate) fn core_ops_eq(
        arguments: Vec<TypedExpression>,
        span: Span,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let call_path = CallPath {
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
        };
        let method_name = MethodName::FromTrait {
            call_path: call_path.clone(),
        };
        let arguments = VecDeque::from(arguments);
        let method = check!(
            resolve_method_name(
                &method_name,
                arguments.clone(),
                vec![],
                span.clone(),
                namespace,
                self_type,
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        instantiate_function_application_simple(
            call_path,
            HashMap::new(),
            arguments,
            method,
            None,
            IsConstant::No,
            span,
        )
    }

    /// If this expression deterministically_aborts 100% of the time, this function returns
    /// `true`. Used in dead-code and control-flow analysis.
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
            SizeOfValue { expr } => expr.deterministically_aborts(),
            StructFieldAccess { .. }
            | Literal(_)
            | StorageAccess { .. }
            | TypeProperty { .. }
            | GenerateUid { .. }
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
            AbiName(_) => false,
            EnumTag { exp } => exp.deterministically_aborts(),
            UnsafeDowncast { exp, .. } => exp.deterministically_aborts(),
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
            | TypedExpressionVariant::SizeOfValue { .. }
            | TypedExpressionVariant::TypeProperty { .. }
            | TypedExpressionVariant::StructExpression { .. }
            | TypedExpressionVariant::VariableExpression { .. }
            | TypedExpressionVariant::AbiName(_)
            | TypedExpressionVariant::StorageAccess { .. }
            | TypedExpressionVariant::FunctionApplication { .. }
            | TypedExpressionVariant::EnumTag { .. }
            | TypedExpressionVariant::UnsafeDowncast { .. }
            | TypedExpressionVariant::GenerateUid { .. } => vec![],
        }
    }

    pub(crate) fn type_check(arguments: TypeCheckArguments<'_, Expression>) -> CompileResult<Self> {
        let TypeCheckArguments {
            checkee: other,
            namespace,
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
                    checkee: (*condition, *then, r#else.map(|x| *x)),
                    return_type_annotation: type_annotation,
                    namespace,
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
                value,
                branches,
                span,
            } => Self::type_check_match_expression(
                TypeCheckArguments {
                    checkee: (*value, branches),
                    return_type_annotation: type_annotation,
                    namespace,
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
                self_type,
                build_config,
                dead_code_graph,
                opts,
            ),
            Expression::StructExpression {
                span,
                type_arguments,
                struct_name,
                fields,
            } => Self::type_check_struct_expression(
                span,
                struct_name,
                type_arguments,
                fields,
                namespace,
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
                *prefix,
                span,
                field_to_access,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
                opts,
            ),
            Expression::MethodApplication {
                method_name,
                contract_call_params,
                arguments,
                type_arguments,
                span,
            } => type_check_method_application(
                method_name,
                contract_call_params,
                arguments,
                type_arguments,
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
                opts,
            ),
            Expression::Tuple { fields, span } => Self::type_check_tuple(
                fields,
                span,
                namespace,
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
                self_type,
                build_config,
                dead_code_graph,
                opts,
            ),
            Expression::Array { contents, span } => Self::type_check_array(
                contents,
                span,
                namespace,
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
            Expression::StorageAccess { field_names, .. } => Self::type_check_storage_load(
                TypeCheckArguments {
                    checkee: field_names,
                    namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    opts,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    mode: Default::default(),
                    help_text: Default::default(),
                },
                &expr_span,
            ),
            Expression::SizeOfVal { exp, span } => Self::type_check_size_of_val(
                TypeCheckArguments {
                    checkee: *exp,
                    namespace,
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
            Expression::BuiltinGetTypeProperty {
                builtin,
                type_name,
                type_span,
                span,
            } => Self::type_check_get_type_property(
                builtin,
                TypeCheckArguments {
                    checkee: (type_name, type_span),
                    namespace,
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
            Expression::BuiltinGenerateUid { span } => ok(
                TypedExpression {
                    expression: TypedExpressionVariant::GenerateUid { span: span.clone() },
                    return_type: insert_type(TypeInfo::B256),
                    is_constant: IsConstant::No,
                    span,
                },
                vec![],
                vec![],
            ),
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
            let (mut new_warnings, new_errors) = unify_with_self(
                typed_expression.return_type,
                type_annotation,
                self_type,
                &expr_span,
                help_text,
            );
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        }

        // The annotation may result in a cast, which is handled in the type engine.
        typed_expression.return_type = check!(
            namespace.resolve_type_with_self(
                look_up_type_id(typed_expression.return_type),
                self_type,
                &expr_span,
                EnforceTypeArguments::No
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

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
        namespace: &Namespace,
    ) -> CompileResult<TypedExpression> {
        let mut errors = vec![];
        let exp = match namespace.resolve_symbol(&name).value {
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
            Some(TypedDeclaration::AbiDeclaration(decl)) => TypedExpression {
                return_type: decl.as_type(),
                is_constant: IsConstant::Yes,
                expression: TypedExpressionVariant::AbiName(AbiName::Known(
                    decl.name.clone().into(),
                )),
                span,
            },
            Some(a) => {
                errors.push(CompileError::NotAVariable {
                    name: name.clone(),
                    what_it_is: a.friendly_name(),
                });
                error_recovery_expr(name.span().clone())
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.clone(),
                });
                error_recovery_expr(name.span().clone())
            }
        };
        ok(exp, vec![], errors)
    }

    #[allow(clippy::type_complexity)]
    fn type_check_function_application(
        arguments: TypeCheckArguments<'_, (CallPath, Vec<Expression>, Vec<TypeArgument>)>,
        _span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (name, arguments, type_arguments),
            namespace,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
        let unknown_decl = check!(
            namespace.resolve_call_path(&name).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let function_decl = check!(
            unknown_decl.expect_function(),
            return err(warnings, errors),
            warnings,
            errors
        );
        instantiate_function_application(
            function_decl.clone(),
            name,
            type_arguments,
            arguments,
            namespace,
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

        let exp = instantiate_lazy_operator(op, typed_lhs, typed_rhs, return_type_annotation, span);
        ok(exp, warnings, errors)
    }

    fn type_check_code_block(
        contents: CodeBlock,
        span: Span,
        namespace: &mut Namespace,
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

        let (mut new_warnings, new_errors) = unify_with_self(
            block_return_type,
            type_annotation,
            self_type,
            &span,
            help_text,
        );
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
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
        arguments: TypeCheckArguments<'_, (Expression, Expression, Option<Expression>)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (condition, then, r#else),
            namespace,
            return_type_annotation: type_annotation,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;
        let condition = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: condition.clone(),
                namespace,
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
        );
        let then = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: then.clone(),
                namespace,
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
        );
        let r#else = r#else.map(|expr| {
            check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: expr.clone(),
                    namespace,
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
            )
        });
        let exp = check!(
            instantiate_if_expression(condition, then, r#else, span, type_annotation, self_type),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_match_expression(
        arguments: TypeCheckArguments<'_, (Expression, Vec<MatchBranch>)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (value, branches),
            namespace,
            return_type_annotation,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            ..
        } = arguments;

        // type check the value
        let typed_value = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: value.clone(),
                namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            }),
            error_recovery_expr(value.span()),
            warnings,
            errors
        );
        let type_id = typed_value.return_type;

        let _ = check!(
            look_up_type_id(type_id).expect_is_supported_in_match_expressions(&typed_value.span),
            return err(warnings, errors),
            warnings,
            errors
        );

        let scrutinees = branches
            .iter()
            .map(|branch| branch.scrutinee.clone())
            .collect::<Vec<_>>();

        // type check the match expression and create a TypedMatchExpression object
        let typed_match_expression = check!(
            TypedMatchExpression::type_check(
                TypeCheckArguments {
                    checkee: (typed_value, branches),
                    namespace,
                    return_type_annotation,
                    help_text: Default::default(),
                    self_type,
                    build_config,
                    dead_code_graph,
                    mode: Mode::NonAbi,
                    opts,
                },
                span.clone()
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check to see if the match expression is exhaustive and if all match arms are reachable
        let (witness_report, arms_reachability) = check!(
            check_match_expression_usefulness(type_id, scrutinees, span.clone()),
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

        // desugar the typed match expression to a typed if expression
        let typed_if_exp = check!(
            typed_match_expression.convert_to_typed_if_expression(namespace, self_type),
            return err(warnings, errors),
            warnings,
            errors
        );

        ok(typed_if_exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        asm: AsmExpression,
        span: Span,
        namespace: &mut Namespace,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let asm_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());
        let return_type = check!(
            namespace.resolve_type_with_self(
                asm.return_type.clone(),
                self_type,
                &asm_span,
                EnforceTypeArguments::No
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
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
        // check for any disallowed opcodes
        for op in &asm.body {
            check!(disallow_opcode(&op.op_name), continue, warnings, errors)
        }
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
        type_arguments: Vec<TypeArgument>,
        fields: Vec<StructExpressionField>,
        namespace: &mut Namespace,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // find the module that the symbol is in
        let module_path = namespace.find_module_path(&call_path.prefixes);
        check!(
            namespace.root().check_submodule(&module_path),
            return err(warnings, errors),
            warnings,
            errors
        );

        // find the struct definition from the name
        let unknown_decl = check!(
            namespace
                .root()
                .resolve_symbol(&module_path, &call_path.suffix)
                .cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let struct_decl = check!(
            unknown_decl.expect_struct(),
            return err(warnings, errors),
            warnings,
            errors
        )
        .clone();

        // monomorphize the struct definition
        let mut struct_decl = check!(
            namespace.monomorphize(
                struct_decl,
                type_arguments,
                EnforceTypeArguments::No,
                Some(self_type),
                None
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // match up the names with their type annotations from the declaration
        let mut typed_fields_buf = vec![];
        for def_field in struct_decl.fields.iter_mut() {
            let expr_field: crate::parse_tree::StructExpressionField =
                match fields.iter().find(|x| x.name == def_field.name) {
                    Some(val) => val.clone(),
                    None => {
                        errors.push(CompileError::StructMissingField {
                            field_name: def_field.name.clone(),
                            struct_name: struct_decl.name.clone(),
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

            def_field.span = typed_field.span.clone();
            typed_fields_buf.push(TypedStructExpressionField {
                value: typed_field,
                name: expr_field.name.clone(),
            });
        }

        // check that there are no extra fields
        for field in fields {
            if !struct_decl.fields.iter().any(|x| x.name == field.name) {
                errors.push(CompileError::StructDoesNotHaveField {
                    field_name: field.name.clone(),
                    struct_name: struct_decl.name.clone(),
                    span: field.span,
                });
            }
        }
        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructExpression {
                struct_name: struct_decl.name.clone(),
                fields: typed_fields_buf,
            },
            return_type: struct_decl.create_type_id(),
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_subfield_expression(
        prefix: Expression,
        span: Span,
        field_to_access: Ident,
        namespace: &mut Namespace,
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
        let exp = check!(
            instantiate_struct_field_access(parent, field_to_access, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_tuple(
        fields: Vec<Expression>,
        span: Span,
        namespace: &mut Namespace,
        type_annotation: TypeId,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph,
        opts: TCOpts,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let field_type_opt = match look_up_type_id(type_annotation) {
            TypeInfo::Tuple(field_type_ids) if field_type_ids.len() == fields.len() => {
                Some(field_type_ids)
            }
            _ => None,
        };
        let mut typed_field_types = Vec::with_capacity(fields.len());
        let mut typed_fields = Vec::with_capacity(fields.len());
        let mut is_constant = IsConstant::Yes;
        for (i, field) in fields.into_iter().enumerate() {
            let field_type = field_type_opt
                .as_ref()
                .map(|field_type_ids| field_type_ids[i].clone())
                .unwrap_or_default();
            let field_span = field.span();
            let typed_field = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: field,
                    namespace,
                    return_type_annotation: field_type.type_id,
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
            typed_field_types.push(TypeArgument {
                type_id: typed_field.return_type,
                span: typed_field.span.clone(),
            });
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

    /// Look up the current global storage state that has been created by storage declarations.
    /// If there isn't any storage, then this is an error. If there is storage, find the corresponding
    /// field that has been specified and return that value.
    fn type_check_storage_load(
        arguments: TypeCheckArguments<'_, Vec<Ident>>,
        span: &Span,
    ) -> CompileResult<TypedExpression> {
        let TypeCheckArguments {
            checkee, namespace, ..
        } = arguments;

        let mut warnings = vec![];
        let mut errors = vec![];
        if !namespace.has_storage_declared() {
            errors.push(CompileError::NoDeclaredStorage { span: span.clone() });
            return err(warnings, errors);
        }

        let storage_fields = check!(
            namespace.get_storage_field_descriptors(),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Do all namespace checking here!
        let (storage_access, return_type) = check!(
            namespace.apply_storage_load(checkee, &storage_fields),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(
            TypedExpression {
                expression: TypedExpressionVariant::StorageAccess(storage_access),
                return_type,
                is_constant: IsConstant::No,
                span: span.clone(),
            },
            warnings,
            errors,
        )
    }

    fn type_check_tuple_index(
        prefix: Expression,
        index: usize,
        index_span: Span,
        span: Span,
        namespace: &mut Namespace,
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
        let exp = check!(
            instantiate_tuple_index_access(parent, index, index_span, span),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_delineated_path(
        call_path: CallPath,
        span: Span,
        args: Vec<Expression>,
        type_arguments: Vec<TypeArgument>,
        namespace: &mut Namespace,
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

        // First, check if this could be a module. We check first so that we can check for
        // ambiguity in the following enum check.
        let is_module = namespace
            .check_submodule(&call_path.prefixes)
            .ok(&mut probe_warnings, &mut probe_errors)
            .is_some();

        // Check if the call path refers to an enum in another module.
        let (enum_name, enum_mod_path) = call_path.prefixes.split_last().expect("empty call path");
        let abs_enum_mod_path: Vec<_> = namespace.find_module_path(enum_mod_path);
        let exp = if let Some(enum_decl) = namespace
            .check_submodule_mut(enum_mod_path)
            .ok(&mut warnings, &mut errors)
            .map(|_| ())
            .and_then(|_| {
                namespace
                    .root()
                    .resolve_symbol(&abs_enum_mod_path, enum_name)
                    .value
            })
            .and_then(|decl| decl.as_enum().cloned())
        {
            // Check for ambiguity between this enum name and a module name.
            if is_module {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            check!(
                instantiate_enum(
                    enum_decl,
                    call_path.suffix,
                    args,
                    type_arguments,
                    namespace,
                    self_type,
                    build_config,
                    dead_code_graph,
                    opts,
                ),
                return err(warnings, errors),
                warnings,
                errors
            )

        // Otherwise, our prefix should point to some module ending with an enum or function.
        } else if namespace
            .check_submodule_mut(&call_path.prefixes)
            .ok(&mut probe_warnings, &mut probe_errors)
            .is_some()
        {
            let decl = check!(
                namespace.resolve_call_path(&call_path).cloned(),
                return err(warnings, errors),
                warnings,
                errors
            );
            match decl {
                TypedDeclaration::EnumDeclaration(enum_decl) => {
                    check!(
                        instantiate_enum(
                            enum_decl,
                            call_path.suffix,
                            args,
                            type_arguments,
                            namespace,
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
                TypedDeclaration::FunctionDeclaration(func_decl) => {
                    check!(
                        instantiate_function_application(
                            func_decl,
                            call_path,
                            vec!(), // the type args in this position are guarenteed to be empty due to parsing
                            args,
                            namespace,
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
                a => {
                    // TODO: Should this be `NotAnEnumOrFunction`?
                    errors.push(CompileError::NotAnEnum {
                        name: call_path.friendly_name(),
                        span,
                        actually: a.friendly_name().to_string(),
                    });
                    return err(warnings, errors);
                }
            }

        // If prefix is neither a module or enum, there's nothing to be found.
        } else {
            errors.push(CompileError::SymbolNotFound {
                name: call_path.suffix.clone(),
            });
            return err(warnings, errors);
        };

        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        abi_name: CallPath,
        address: Box<Expression>,
        span: Span,
        namespace: &mut Namespace,
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
        let address_expr = check!(
            TypedExpression::type_check(TypeCheckArguments {
                checkee: *address,
                namespace,
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
            namespace.resolve_call_path(&abi_name).cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );
        let abi = match abi {
            TypedDeclaration::AbiDeclaration(abi) => abi,
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                body: ref expr,
                ..
            }) => {
                let ret_ty = look_up_type_id(expr.return_type);
                let abi_name = match ret_ty {
                    TypeInfo::ContractCaller { abi_name, .. } => abi_name,
                    _ => {
                        errors.push(CompileError::NotAnAbi {
                            span: abi_name.span(),
                            actually_is: abi.friendly_name(),
                        });
                        return err(warnings, errors);
                    }
                };
                match abi_name {
                    // look up the call path and get the declaration it references
                    AbiName::Known(abi_name) => {
                        let unknown_decl = check!(
                            namespace.resolve_call_path(&abi_name).cloned(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        );
                        check!(
                            unknown_decl.expect_abi(),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                        .clone()
                    }
                    AbiName::Deferred => {
                        return ok(
                            TypedExpression {
                                return_type: insert_type(TypeInfo::ContractCaller {
                                    abi_name: AbiName::Deferred,
                                    address: None,
                                }),
                                expression: TypedExpressionVariant::Tuple { fields: vec![] },
                                is_constant: IsConstant::Yes,
                                span,
                            },
                            warnings,
                            errors,
                        )
                    }
                }
            }
            a => {
                errors.push(CompileError::NotAnAbi {
                    span: abi_name.span(),
                    actually_is: a.friendly_name(),
                });
                return err(warnings, errors);
            }
        };

        let return_type = insert_type(TypeInfo::ContractCaller {
            abi_name: AbiName::Known(abi_name.clone()),
            address: Some(Box::new(address_expr.clone())),
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
        namespace: &mut Namespace,
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
            let (mut new_warnings, new_errors) = unify_with_self(
                typed_elem.return_type,
                elem_type,
                self_type,
                &typed_elem.span,
                "",
            );
            let no_warnings = new_warnings.is_empty();
            let no_errors = new_errors.is_empty();
            warnings.append(&mut new_warnings);
            errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
            // In both cases, if there are warnings or errors then break here, since we don't
            // need to spam type errors for every element once we have one.
            if !no_warnings && !no_errors {
                break;
            }
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
            let method_name = MethodName::FromTrait {
                call_path: CallPath {
                    prefixes: vec![
                        Ident::new_with_override("core", span.clone()),
                        Ident::new_with_override("ops", span.clone()),
                    ],
                    suffix: Ident::new_with_override("index", span.clone()),
                    is_absolute: true,
                },
            };
            type_check_method_application(
                method_name,
                vec![],
                vec![prefix, index],
                vec![],
                span,
                namespace,
                self_type,
                build_config,
                dead_code_graph,
                opts,
            )
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
            expression: TypedExpressionVariant::SizeOfValue {
                expr: Box::new(exp),
            },
            return_type: crate::type_engine::insert_type(TypeInfo::UnsignedInteger(
                IntegerBits::SixtyFour,
            )),
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_get_type_property(
        builtin: BuiltinProperty,
        arguments: TypeCheckArguments<'_, (TypeInfo, Span)>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let TypeCheckArguments {
            checkee: (type_name, type_span),
            self_type,
            namespace,
            ..
        } = arguments;
        let type_id = check!(
            namespace.resolve_type_with_self(
                type_name,
                self_type,
                &type_span,
                EnforceTypeArguments::Yes
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );
        let return_type = match builtin {
            BuiltinProperty::SizeOfType => {
                insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour))
            }
            BuiltinProperty::IsRefType => insert_type(TypeInfo::Boolean),
        };
        let exp = TypedExpression {
            expression: TypedExpressionVariant::TypeProperty {
                property: builtin,
                type_id,
                span: span.clone(),
            },
            return_type,
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

        // Parse and resolve a Numeric(span) based on new_type.
        let (val, new_integer_type) = match lit {
            Literal::Numeric(num) => match look_up_type_id(new_type) {
                TypeInfo::UnsignedInteger(n) => match n {
                    IntegerBits::Eight => (
                        num.to_string().parse().map(Literal::U8).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Eight),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::Sixteen => (
                        num.to_string().parse().map(Literal::U16).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::ThirtyTwo => (
                        num.to_string().parse().map(Literal::U32).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                                span.clone(),
                            )
                        }),
                        new_type,
                    ),
                    IntegerBits::SixtyFour => (
                        num.to_string().parse().map(Literal::U64).map_err(|e| {
                            Literal::handle_parse_int_error(
                                e,
                                TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                                span.clone(),
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
                            span.clone(),
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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    fn do_type_check(expr: Expression, type_annotation: TypeId) -> CompileResult<TypedExpression> {
        let mut namespace = Namespace::init_root(namespace::Module::default());
        let self_type = insert_type(TypeInfo::Unknown);
        let build_config = BuildConfig {
            file_name: Arc::new("test.sw".into()),
            dir_of_code: Arc::new("".into()),
            manifest_path: Arc::new("".into()),
            use_orig_asm: false,
            print_intermediate_asm: false,
            print_finalized_asm: false,
            print_ir: false,
            generated_names: Arc::new(Mutex::new(vec![])),
        };
        let mut dead_code_graph: ControlFlowGraph = Default::default();

        TypedExpression::type_check(TypeCheckArguments {
            checkee: expr,
            namespace: &mut namespace,
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
        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: Span::dummy(),
                },
                Expression::Literal {
                    value: Literal::U64(0),
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
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
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::U64(0),
                    span: Span::dummy(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
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
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression::Array {
            contents: vec![
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: Span::dummy(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: Span::dummy(),
                },
                Expression::Literal {
                    value: Literal::Boolean(true),
                    span: Span::dummy(),
                },
            ],
            span: Span::dummy(),
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
        let expr = Expression::Array {
            contents: Vec::new(),
            span: Span::dummy(),
        };

        let comp_res = do_type_check(
            expr,
            insert_type(TypeInfo::Array(insert_type(TypeInfo::Boolean), 0)),
        );
        assert!(comp_res.warnings.is_empty() && comp_res.errors.is_empty());
    }
}
fn disallow_opcode(op: &Ident) -> CompileResult<()> {
    let mut errors = vec![];

    match op.as_str().to_lowercase().as_str() {
        "ji" => {
            errors.push(CompileError::DisallowedJi {
                span: op.span().clone(),
            });
        }
        "jnei" => {
            errors.push(CompileError::DisallowedJnei {
                span: op.span().clone(),
            });
        }
        "jnzi" => {
            errors.push(CompileError::DisallowedJnzi {
                span: op.span().clone(),
            });
        }
        _ => (),
    };
    if errors.is_empty() {
        ok((), vec![], vec![])
    } else {
        err(vec![], errors)
    }
}
