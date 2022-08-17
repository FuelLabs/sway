mod enum_instantiation;
mod function_application;
mod if_expression;
mod lazy_operator;
mod method_application;
mod struct_field_access;
mod tuple_index_access;
mod unsafe_downcast;

pub(crate) use self::{
    enum_instantiation::*, function_application::*, if_expression::*, lazy_operator::*,
    method_application::*, struct_field_access::*, tuple_index_access::*, unsafe_downcast::*,
};

use crate::{
    error::*, parse_tree::*, semantic_analysis::*, type_system::*, types::DeterministicallyAborts,
};

use sway_ast::intrinsics::Intrinsic;
use sway_types::{Ident, Span, Spanned};

use std::{
    collections::{HashMap, VecDeque},
    fmt,
};

#[derive(Clone, Debug, Eq)]
pub struct TypedExpression {
    pub expression: TypedExpressionVariant,
    pub return_type: TypeId,
    /// whether or not this expression is constantly evaluable (if the result is known at compile
    /// time)
    pub(crate) is_constant: IsConstant,
    pub span: Span,
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

impl fmt::Display for TypedExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.expression,
            look_up_type_id(self.return_type)
        )
    }
}

impl CollectTypesMetadata for TypedExpression {
    fn collect_types_metadata(&self) -> Vec<TypeMetadata> {
        use TypedExpressionVariant::*;
        let mut res = self.return_type.collect_types_metadata();
        match &self.expression {
            FunctionApplication {
                arguments,
                function_decl,
                ..
            } => {
                res.append(
                    &mut arguments
                        .iter()
                        .map(|x| &x.1)
                        .flat_map(CollectTypesMetadata::collect_types_metadata)
                        .collect::<Vec<_>>(),
                );
                res.append(
                    &mut function_decl
                        .body
                        .contents
                        .iter()
                        .flat_map(CollectTypesMetadata::collect_types_metadata)
                        .collect(),
                );
            }
            Tuple { fields } => {
                res.append(
                    &mut fields
                        .iter()
                        .flat_map(|x| x.collect_types_metadata())
                        .collect(),
                );
            }
            AsmExpression { registers, .. } => {
                res.append(
                    &mut registers
                        .iter()
                        .filter_map(|x| x.initializer.as_ref())
                        .flat_map(CollectTypesMetadata::collect_types_metadata)
                        .collect::<Vec<_>>(),
                );
            }
            StructExpression { fields, .. } => {
                res.append(
                    &mut fields
                        .iter()
                        .flat_map(|x| x.value.collect_types_metadata())
                        .collect(),
                );
            }
            LazyOperator { lhs, rhs, .. } => {
                res.append(&mut lhs.collect_types_metadata());
                res.append(&mut rhs.collect_types_metadata());
            }
            Array { contents } => {
                res.append(
                    &mut contents
                        .iter()
                        .flat_map(|x| x.collect_types_metadata())
                        .collect(),
                );
            }
            ArrayIndex { prefix, index } => {
                res.append(&mut prefix.collect_types_metadata());
                res.append(&mut index.collect_types_metadata());
            }
            CodeBlock(block) => {
                res.append(
                    &mut block
                        .contents
                        .iter()
                        .flat_map(CollectTypesMetadata::collect_types_metadata)
                        .collect(),
                );
            }
            IfExp {
                condition,
                then,
                r#else,
            } => {
                res.append(&mut condition.collect_types_metadata());
                res.append(&mut then.collect_types_metadata());
                if let Some(r#else) = r#else {
                    res.append(&mut r#else.collect_types_metadata());
                }
            }
            StructFieldAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata());
                res.append(&mut resolved_type_of_parent.collect_types_metadata());
            }
            TupleElemAccess {
                prefix,
                resolved_type_of_parent,
                ..
            } => {
                res.append(&mut prefix.collect_types_metadata());
                res.append(&mut resolved_type_of_parent.collect_types_metadata());
            }
            EnumInstantiation {
                enum_decl,
                contents,
                ..
            } => {
                if let Some(contents) = contents {
                    res.append(&mut contents.collect_types_metadata().into_iter().collect());
                }
                res.append(
                    &mut enum_decl
                        .variants
                        .iter()
                        .flat_map(|x| x.type_id.collect_types_metadata())
                        .collect(),
                );
                res.append(
                    &mut enum_decl
                        .type_parameters
                        .iter()
                        .flat_map(|x| x.type_id.collect_types_metadata())
                        .collect(),
                );
            }
            AbiCast { address, .. } => {
                res.append(&mut address.collect_types_metadata());
            }
            IntrinsicFunction(kind) => {
                res.append(&mut kind.collect_types_metadata());
            }
            EnumTag { exp } => {
                res.append(&mut exp.collect_types_metadata());
            }
            UnsafeDowncast { exp, variant } => {
                res.append(&mut exp.collect_types_metadata());
                res.append(&mut variant.type_id.collect_types_metadata());
            }
            WhileLoop { condition, body } => {
                res.append(&mut condition.collect_types_metadata());
                res.append(
                    &mut body
                        .contents
                        .iter()
                        .flat_map(TypedAstNode::collect_types_metadata)
                        .collect(),
                );
            }
            Return(stmt) => res.append(&mut stmt.expr.collect_types_metadata()),
            // storage access can never be generic
            // variable expressions don't ever have return types themselves, they're stored in
            // `TypedExpression::return_type`. Variable expressions are just names of variables.
            VariableExpression { .. }
            | StorageAccess { .. }
            | Literal(_)
            | AbiName(_)
            | Break
            | Continue
            | FunctionParameter => {}
            Reassignment(reassignment) => {
                res.append(&mut reassignment.rhs.collect_types_metadata())
            }
            StorageReassignment(storage_reassignment) => res.extend(
                storage_reassignment
                    .fields
                    .iter()
                    .flat_map(|x| x.type_id.collect_types_metadata())
                    .chain(
                        storage_reassignment
                            .rhs
                            .collect_types_metadata()
                            .into_iter(),
                    ),
            ),
        }
        res
    }
}

impl DeterministicallyAborts for TypedExpression {
    fn deterministically_aborts(&self) -> bool {
        use TypedExpressionVariant::*;
        match &self.expression {
            FunctionApplication {
                function_decl,
                arguments,
                ..
            } => {
                function_decl.body.deterministically_aborts()
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
            StructFieldAccess { .. }
            | Literal(_)
            | StorageAccess { .. }
            | VariableExpression { .. }
            | FunctionParameter
            | TupleElemAccess { .. } => false,
            IntrinsicFunction(kind) => kind.deterministically_aborts(),
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
            WhileLoop { condition, body } => {
                condition.deterministically_aborts() || body.deterministically_aborts()
            }
            Break => false,
            Continue => false,
            Reassignment(reassignment) => reassignment.rhs.deterministically_aborts(),
            StorageReassignment(storage_reassignment) => {
                storage_reassignment.rhs.deterministically_aborts()
            }
            // TODO: Is this correct?
            // I'm not sure what this function is supposed to do exactly. It's called
            // "deterministically_aborts" which I thought meant it checks for an abort/panic, but
            // it's actually checking for returns.
            //
            // Also, is it necessary to check the expression to see if avoids the return? eg.
            // someone could write `return break;` in a loop, which would mean the return never
            // gets executed.
            Return(..) => true,
        }
    }
}

pub(crate) fn error_recovery_expr(span: Span) -> TypedExpression {
    TypedExpression {
        expression: TypedExpressionVariant::Tuple { fields: vec![] },
        return_type: crate::type_system::insert_type(TypeInfo::ErrorRecovery),
        is_constant: IsConstant::No,
        span,
    }
}

#[allow(clippy::too_many_arguments)]
impl TypedExpression {
    pub(crate) fn core_ops_eq(
        ctx: TypeCheckContext,
        arguments: Vec<TypedExpression>,
        span: Span,
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
        let method_name_binding = TypeBinding {
            inner: MethodName::FromTrait {
                call_path: call_path.clone(),
            },
            type_arguments: vec![],
            span: call_path.span(),
        };
        let arguments = VecDeque::from(arguments);
        let method = check!(
            resolve_method_name(ctx, &method_name_binding, arguments.clone()),
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
            None,
            span,
        )
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
            TypedExpressionVariant::WhileLoop { condition, body } => {
                let mut buf = condition.gather_return_statements();
                for node in &body.contents {
                    buf.append(&mut node.gather_return_statements())
                }
                buf
            }
            TypedExpressionVariant::Reassignment(reassignment) => {
                reassignment.rhs.gather_return_statements()
            }
            TypedExpressionVariant::StorageReassignment(storage_reassignment) => {
                storage_reassignment.rhs.gather_return_statements()
            }
            TypedExpressionVariant::LazyOperator { lhs, rhs, .. } => [lhs, rhs]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::Tuple { fields } => fields
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::Array { contents } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::ArrayIndex { prefix, index } => [prefix, index]
                .into_iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::StructFieldAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TypedExpressionVariant::TupleElemAccess { prefix, .. } => {
                prefix.gather_return_statements()
            }
            TypedExpressionVariant::EnumInstantiation { contents, .. } => contents
                .iter()
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::AbiCast { address, .. } => address.gather_return_statements(),
            TypedExpressionVariant::IntrinsicFunction(intrinsic_function_kind) => {
                intrinsic_function_kind
                    .arguments
                    .iter()
                    .flat_map(|expr| expr.gather_return_statements())
                    .collect()
            }
            TypedExpressionVariant::StructExpression { fields, .. } => fields
                .iter()
                .flat_map(|field| field.value.gather_return_statements())
                .collect(),
            TypedExpressionVariant::FunctionApplication {
                contract_call_params,
                arguments,
                selector,
                ..
            } => contract_call_params
                .values()
                .chain(arguments.iter().map(|(_name, expr)| expr))
                .chain(
                    selector
                        .iter()
                        .map(|contract_call_params| &*contract_call_params.contract_address),
                )
                .flat_map(|expr| expr.gather_return_statements())
                .collect(),
            TypedExpressionVariant::EnumTag { exp } => exp.gather_return_statements(),
            TypedExpressionVariant::UnsafeDowncast { exp, .. } => exp.gather_return_statements(),

            TypedExpressionVariant::Return(stmt) => {
                vec![stmt]
            }
            // if it is impossible for an expression to contain a return _statement_ (not an
            // implicit return!), put it in the pattern below.
            TypedExpressionVariant::Literal(_)
            | TypedExpressionVariant::FunctionParameter { .. }
            | TypedExpressionVariant::AsmExpression { .. }
            | TypedExpressionVariant::VariableExpression { .. }
            | TypedExpressionVariant::AbiName(_)
            | TypedExpressionVariant::StorageAccess { .. }
            | TypedExpressionVariant::Break
            | TypedExpressionVariant::Continue => vec![],
        }
    }

    /// gathers the mutability of the expressions within
    pub(crate) fn gather_mutability(&self) -> VariableMutability {
        match &self.expression {
            TypedExpressionVariant::VariableExpression { mutability, .. } => *mutability,
            _ => VariableMutability::Immutable,
        }
    }

    pub(crate) fn type_check(mut ctx: TypeCheckContext, expr: Expression) -> CompileResult<Self> {
        let expr_span = expr.span();
        let span = expr_span.clone();
        let res = match expr.kind {
            ExpressionKind::Literal(lit) => Self::type_check_literal(lit, span),
            ExpressionKind::Variable(name) => {
                Self::type_check_variable_expression(ctx.namespace, name, span)
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                let FunctionApplicationExpression {
                    call_path_binding,
                    arguments,
                } = *function_application_expression;
                Self::type_check_function_application(
                    ctx.by_ref(),
                    call_path_binding,
                    arguments,
                    span,
                )
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { op, lhs, rhs }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Boolean));
                Self::type_check_lazy_operator(ctx, op, *lhs, *rhs, span)
            }
            ExpressionKind::CodeBlock(contents) => {
                Self::type_check_code_block(ctx.by_ref(), contents, span)
            }
            // TODO if _condition_ is constant, evaluate it and compile this to an
            // expression with only one branch
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
            }) => Self::type_check_if_expression(
                ctx.by_ref().with_help_text(""),
                *condition,
                *then,
                r#else.map(|e| *e),
                span,
            ),
            ExpressionKind::Match(MatchExpression { value, branches }) => {
                Self::type_check_match_expression(
                    ctx.by_ref().with_help_text(""),
                    *value,
                    branches,
                    span,
                )
            }
            ExpressionKind::Asm(asm) => Self::type_check_asm_expression(ctx.by_ref(), *asm, span),
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    fields,
                } = *struct_expression;
                Self::type_check_struct_expression(ctx.by_ref(), call_path_binding, fields, span)
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
            }) => {
                Self::type_check_subfield_expression(ctx.by_ref(), *prefix, span, field_to_access)
            }
            ExpressionKind::MethodApplication(method_application_expression) => {
                let MethodApplicationExpression {
                    method_name_binding,
                    contract_call_params,
                    arguments,
                } = *method_application_expression;
                type_check_method_application(
                    ctx.by_ref(),
                    method_name_binding,
                    contract_call_params,
                    arguments,
                    span,
                )
            }
            ExpressionKind::Tuple(fields) => Self::type_check_tuple(ctx.by_ref(), fields, span),
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix,
                index,
                index_span,
            }) => Self::type_check_tuple_index(ctx.by_ref(), *prefix, index, index_span, span),
            ExpressionKind::DelineatedPath(delineated_path_expression) => {
                let DelineatedPathExpression {
                    call_path_binding,
                    args,
                } = *delineated_path_expression;
                Self::type_check_delineated_path(ctx.by_ref(), call_path_binding, span, args)
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                let AbiCastExpression { abi_name, address } = *abi_cast_expression;
                Self::type_check_abi_cast(ctx.by_ref(), abi_name, *address, span)
            }
            ExpressionKind::Array(contents) => Self::type_check_array(ctx.by_ref(), contents, span),
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_array_index(ctx, *prefix, *index, span)
            }
            ExpressionKind::StorageAccess(StorageAccessExpression { field_names }) => {
                let ctx = ctx
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                Self::type_check_storage_load(ctx, field_names, &span)
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                kind_binding,
                arguments,
            }) => Self::type_check_intrinsic_function(ctx.by_ref(), kind_binding, arguments, span),
            ExpressionKind::WhileLoop(WhileLoopExpression { condition, body }) => {
                Self::type_check_while_loop(ctx.by_ref(), *condition, body, span)
            }
            ExpressionKind::Break => {
                let expr = TypedExpression {
                    expression: TypedExpressionVariant::Break,
                    return_type: insert_type(TypeInfo::Unknown),
                    is_constant: IsConstant::No,
                    span,
                };
                ok(expr, vec![], vec![])
            }
            ExpressionKind::Continue => {
                let expr = TypedExpression {
                    expression: TypedExpressionVariant::Continue,
                    return_type: insert_type(TypeInfo::Unknown),
                    is_constant: IsConstant::No,
                    span,
                };
                ok(expr, vec![], vec![])
            }
            ExpressionKind::Reassignment(ReassignmentExpression { lhs, rhs }) => {
                Self::type_check_reassignment(ctx.by_ref(), lhs, *rhs, span)
            }
            ExpressionKind::Return(expr) => {
                let ctx = ctx
                    // we use "unknown" here because return statements do not
                    // necessarily follow the type annotation of their immediate
                    // surrounding context. Because a return statement is control flow
                    // that breaks out to the nearest function, we need to type check
                    // it against the surrounding function.
                    // That is impossible here, as we don't have that information. It
                    // is the responsibility of the function declaration to type check
                    // all return statements contained within it.
                    .by_ref()
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text(
                        "Returned value must match up with the function return type \
                        annotation.",
                    );
                let mut warnings = vec![];
                let mut errors = vec![];
                let expr_span = expr.span();
                let expr = check!(
                    TypedExpression::type_check(ctx, *expr),
                    error_recovery_expr(expr_span),
                    warnings,
                    errors,
                );
                let stmt = TypedReturnStatement { expr };
                let typed_expr = TypedExpression {
                    expression: TypedExpressionVariant::Return(Box::new(stmt)),
                    return_type: insert_type(TypeInfo::Unknown),
                    // FIXME: This should be Yes?
                    is_constant: IsConstant::No,
                    span,
                };
                ok(typed_expr, warnings, errors)
            }
        };
        let mut typed_expression = match res.value {
            Some(r) => r,
            None => return res,
        };
        let mut warnings = res.warnings;
        let mut errors = res.errors;

        // if the return type cannot be cast into the annotation type then it is a type error
        let (mut new_warnings, new_errors) =
            ctx.unify_with_self(typed_expression.return_type, &expr_span);
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());

        // The annotation may result in a cast, which is handled in the type engine.
        typed_expression.return_type = check!(
            ctx.resolve_type_with_self(
                typed_expression.return_type,
                &expr_span,
                EnforceTypeArguments::No,
                None
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
        let id = crate::type_system::insert_type(return_type);
        let exp = TypedExpression {
            expression: TypedExpressionVariant::Literal(lit),
            return_type: id,
            is_constant: IsConstant::Yes,
            span,
        };
        ok(exp, vec![], vec![])
    }

    pub(crate) fn type_check_variable_expression(
        namespace: &Namespace,
        name: Ident,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut errors = vec![];
        let exp = match namespace.resolve_symbol(&name).value {
            Some(TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                name: decl_name,
                body,
                mutability,
                ..
            })) => TypedExpression {
                return_type: body.return_type,
                is_constant: body.is_constant,
                expression: TypedExpressionVariant::VariableExpression {
                    name: decl_name.clone(),
                    span: name.span(),
                    mutability: *mutability,
                },
                span,
            },
            Some(TypedDeclaration::ConstantDeclaration(TypedConstantDeclaration {
                name: decl_name,
                value,
                ..
            })) => TypedExpression {
                return_type: value.return_type,
                is_constant: IsConstant::Yes,
                // Although this isn't strictly a 'variable' expression we can treat it as one for
                // this context.
                expression: TypedExpressionVariant::VariableExpression {
                    name: decl_name.clone(),
                    span: name.span(),
                    mutability: VariableMutability::Immutable,
                },
                span,
            },
            Some(TypedDeclaration::AbiDeclaration(decl)) => TypedExpression {
                return_type: decl.create_type_id(),
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
                error_recovery_expr(name.span())
            }
            None => {
                errors.push(CompileError::UnknownVariable {
                    var_name: name.clone(),
                });
                error_recovery_expr(name.span())
            }
        };
        ok(exp, vec![], errors)
    }

    fn type_check_function_application(
        ctx: TypeCheckContext,
        mut call_path_binding: TypeBinding<CallPath>,
        arguments: Vec<Expression>,
        _span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // type deck the declaration
        let unknown_decl = check!(
            TypeBinding::type_check_with_ident(&mut call_path_binding, &ctx),
            return err(warnings, errors),
            warnings,
            errors
        );

        // check that the decl is a function decl
        let function_decl = check!(
            unknown_decl.expect_function().cloned(),
            return err(warnings, errors),
            warnings,
            errors
        );

        instantiate_function_application(ctx, function_decl, call_path_binding.inner, arguments)
    }

    fn type_check_lazy_operator(
        ctx: TypeCheckContext,
        op: LazyOp,
        lhs: Expression,
        rhs: Expression,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut ctx = ctx.with_help_text("");
        let typed_lhs = check!(
            TypedExpression::type_check(ctx.by_ref(), lhs.clone()),
            error_recovery_expr(lhs.span()),
            warnings,
            errors
        );

        let typed_rhs = check!(
            TypedExpression::type_check(ctx.by_ref(), rhs.clone()),
            error_recovery_expr(rhs.span()),
            warnings,
            errors
        );

        let type_annotation = ctx.type_annotation();
        let exp = instantiate_lazy_operator(op, typed_lhs, typed_rhs, type_annotation, span);
        ok(exp, warnings, errors)
    }

    fn type_check_code_block(
        mut ctx: TypeCheckContext,
        contents: CodeBlock,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (typed_block, block_return_type) = check!(
            TypedCodeBlock::type_check(ctx.by_ref(), contents),
            (
                TypedCodeBlock { contents: vec![] },
                crate::type_system::insert_type(TypeInfo::Tuple(Vec::new()))
            ),
            warnings,
            errors
        );

        let (mut new_warnings, new_errors) = ctx.unify_with_self(block_return_type, &span);
        warnings.append(&mut new_warnings);
        errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());
        let exp = TypedExpression {
            expression: TypedExpressionVariant::CodeBlock(TypedCodeBlock {
                contents: typed_block.contents,
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
        mut ctx: TypeCheckContext,
        condition: Expression,
        then: Expression,
        r#else: Option<Expression>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let condition = {
            let ctx = ctx
                .by_ref()
                .with_help_text("The condition of an if expression must be a boolean expression.")
                .with_type_annotation(insert_type(TypeInfo::Boolean));
            check!(
                TypedExpression::type_check(ctx, condition.clone()),
                error_recovery_expr(condition.span()),
                warnings,
                errors
            )
        };
        let then = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                TypedExpression::type_check(ctx, then.clone()),
                error_recovery_expr(then.span()),
                warnings,
                errors
            )
        };
        let r#else = r#else.map(|expr| {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                TypedExpression::type_check(ctx, expr.clone()),
                error_recovery_expr(expr.span()),
                warnings,
                errors
            )
        });
        let exp = check!(
            instantiate_if_expression(
                condition,
                then,
                r#else,
                span,
                ctx.type_annotation(),
                ctx.self_type(),
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        ok(exp, warnings, errors)
    }

    fn type_check_match_expression(
        mut ctx: TypeCheckContext,
        value: Expression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // type check the value
        let typed_value = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                TypedExpression::type_check(ctx, value.clone()),
                error_recovery_expr(value.span()),
                warnings,
                errors
            )
        };
        let type_id = typed_value.return_type;

        check!(
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
        let typed_match_expression = {
            let ctx = ctx.by_ref().with_help_text("");
            check!(
                TypedMatchExpression::type_check(ctx, typed_value, branches, span.clone()),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

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
            typed_match_expression.convert_to_typed_if_expression(ctx),
            return err(warnings, errors),
            warnings,
            errors
        );

        ok(typed_if_exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_asm_expression(
        mut ctx: TypeCheckContext,
        asm: AsmExpression,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let asm_span = asm
            .returns
            .clone()
            .map(|x| x.1)
            .unwrap_or_else(|| asm.whole_block_span.clone());
        let diverges = asm
            .body
            .iter()
            .any(|asm_op| matches!(asm_op.op_name.as_str(), "rvrt" | "ret"));
        let return_type = if diverges {
            insert_type(TypeInfo::Unknown)
        } else {
            check!(
                ctx.resolve_type_with_self(
                    insert_type(asm.return_type.clone()),
                    &asm_span,
                    EnforceTypeArguments::No,
                    None
                ),
                insert_type(TypeInfo::ErrorRecovery),
                warnings,
                errors,
            )
        };
        // type check the initializers
        let typed_registers = asm
            .registers
            .into_iter()
            .map(
                |AsmRegisterDeclaration { name, initializer }| TypedAsmRegisterDeclaration {
                    name,
                    initializer: initializer.map(|initializer| {
                        let ctx = ctx
                            .by_ref()
                            .with_help_text("")
                            .with_type_annotation(insert_type(TypeInfo::Unknown));
                        check!(
                            TypedExpression::type_check(ctx, initializer.clone()),
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
        mut ctx: TypeCheckContext,
        call_path_binding: TypeBinding<CallPath<(TypeInfo, Span)>>,
        fields: Vec<StructExpressionField>,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // type check the call path
        let type_id = check!(
            call_path_binding.type_check_with_type_info(&mut ctx),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        // extract the struct name and fields from the type info
        let type_info = look_up_type_id(type_id);
        let (struct_name, struct_fields) = check!(
            type_info.expect_struct(&span),
            return err(warnings, errors),
            warnings,
            errors
        );
        let mut struct_fields = struct_fields.clone();

        // match up the names with their type annotations from the declaration
        let mut typed_fields_buf = vec![];
        for def_field in struct_fields.iter_mut() {
            let expr_field: crate::parse_tree::StructExpressionField =
                match fields.iter().find(|x| x.name == def_field.name) {
                    Some(val) => val.clone(),
                    None => {
                        errors.push(CompileError::StructMissingField {
                            field_name: def_field.name.clone(),
                            struct_name: struct_name.clone(),
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

            let ctx = ctx
                .by_ref()
                .with_help_text(
                    "Struct field's type must match up with the type specified in its declaration.",
                )
                .with_type_annotation(def_field.type_id);
            let typed_field = check!(
                TypedExpression::type_check(ctx, expr_field.value),
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
            if !struct_fields.iter().any(|x| x.name == field.name) {
                errors.push(CompileError::StructDoesNotHaveField {
                    field_name: field.name.clone(),
                    struct_name: struct_name.clone(),
                    span: field.span,
                });
            }
        }
        let exp = TypedExpression {
            expression: TypedExpressionVariant::StructExpression {
                struct_name: struct_name.clone(),
                fields: typed_fields_buf,
                span: call_path_binding.inner.suffix.1.clone(),
            },
            return_type: type_id,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_subfield_expression(
        ctx: TypeCheckContext,
        prefix: Expression,
        span: Span,
        field_to_access: Ident,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        let parent = check!(
            TypedExpression::type_check(ctx, prefix),
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
        mut ctx: TypeCheckContext,
        fields: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let field_type_opt = match look_up_type_id(ctx.type_annotation()) {
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
            let ctx = ctx
                .by_ref()
                .with_help_text("tuple field type does not match the expected type")
                .with_type_annotation(field_type.type_id);
            let typed_field = check!(
                TypedExpression::type_check(ctx, field),
                error_recovery_expr(field_span),
                warnings,
                errors
            );
            if let IsConstant::No = typed_field.is_constant {
                is_constant = IsConstant::No;
            }
            typed_field_types.push(TypeArgument {
                type_id: typed_field.return_type,
                initial_type_id: field_type.type_id,
                span: typed_field.span.clone(),
            });
            typed_fields.push(typed_field);
        }
        let exp = TypedExpression {
            expression: TypedExpressionVariant::Tuple {
                fields: typed_fields,
            },
            return_type: crate::type_system::insert_type(TypeInfo::Tuple(typed_field_types)),
            is_constant,
            span,
        };
        ok(exp, warnings, errors)
    }

    /// Look up the current global storage state that has been created by storage declarations.
    /// If there isn't any storage, then this is an error. If there is storage, find the corresponding
    /// field that has been specified and return that value.
    fn type_check_storage_load(
        ctx: TypeCheckContext,
        checkee: Vec<Ident>,
        span: &Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        if !ctx.namespace.has_storage_declared() {
            errors.push(CompileError::NoDeclaredStorage { span: span.clone() });
            return err(warnings, errors);
        }

        let storage_fields = check!(
            ctx.namespace.get_storage_field_descriptors(span),
            return err(warnings, errors),
            warnings,
            errors
        );

        // Do all namespace checking here!
        let (storage_access, return_type) = check!(
            ctx.namespace
                .apply_storage_load(checkee, &storage_fields, span),
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
        ctx: TypeCheckContext,
        prefix: Expression,
        index: usize,
        index_span: Span,
        span: Span,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let ctx = ctx
            .with_help_text("")
            .with_type_annotation(insert_type(TypeInfo::Unknown));
        let parent = check!(
            TypedExpression::type_check(ctx, prefix),
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

    fn type_check_delineated_path(
        ctx: TypeCheckContext,
        call_path_binding: TypeBinding<CallPath>,
        span: Span,
        args: Vec<Expression>,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // The first step is to determine if the call path refers to a module, enum, or function.
        // If only one exists, then we use that one. Otherwise, if more than one exist, it is
        // an ambiguous reference error.

        // Check if this could be a module
        let mut module_probe_warnings = Vec::new();
        let mut module_probe_errors = Vec::new();
        let is_module = {
            let call_path_binding = call_path_binding.clone();
            ctx.namespace
                .check_submodule(
                    &[
                        call_path_binding.inner.prefixes,
                        vec![call_path_binding.inner.suffix],
                    ]
                    .concat(),
                )
                .ok(&mut module_probe_warnings, &mut module_probe_errors)
                .is_some()
        };

        // Check if this could be a function
        let mut function_probe_warnings = Vec::new();
        let mut function_probe_errors = Vec::new();
        let maybe_function = {
            let mut call_path_binding = call_path_binding.clone();
            TypeBinding::type_check_with_ident(&mut call_path_binding, &ctx)
                .flat_map(|unknown_decl| unknown_decl.expect_function().cloned())
                .ok(&mut function_probe_warnings, &mut function_probe_errors)
        };

        // Check if this could be an enum
        let mut enum_probe_warnings = vec![];
        let mut enum_probe_errors = vec![];
        let maybe_enum = {
            let call_path_binding = call_path_binding.clone();
            let enum_name = call_path_binding.inner.prefixes[0].clone();
            let variant_name = call_path_binding.inner.suffix.clone();
            let enum_call_path = call_path_binding.inner.rshift();
            let mut call_path_binding = TypeBinding {
                inner: enum_call_path,
                type_arguments: call_path_binding.type_arguments,
                span: call_path_binding.span,
            };
            TypeBinding::type_check_with_ident(&mut call_path_binding, &ctx)
                .flat_map(|unknown_decl| unknown_decl.expect_enum().cloned())
                .ok(&mut enum_probe_warnings, &mut enum_probe_errors)
                .map(|enum_decl| (enum_decl, enum_name, variant_name))
        };

        // compare the results of the checks
        let exp = match (is_module, maybe_function, maybe_enum) {
            (false, None, Some((enum_decl, enum_name, variant_name))) => {
                warnings.append(&mut enum_probe_warnings);
                errors.append(&mut enum_probe_errors);
                check!(
                    instantiate_enum(ctx, enum_decl, enum_name, variant_name, args),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (false, Some(func_decl), None) => {
                warnings.append(&mut function_probe_warnings);
                errors.append(&mut function_probe_errors);
                check!(
                    instantiate_function_application(ctx, func_decl, call_path_binding.inner, args,),
                    return err(warnings, errors),
                    warnings,
                    errors
                )
            }
            (true, None, None) => {
                module_probe_errors.push(CompileError::Unimplemented(
                    "this case is not yet implemented",
                    span,
                ));
                return err(module_probe_warnings, module_probe_errors);
            }
            (true, None, Some(_)) => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            (true, Some(_), None) => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            (true, Some(_), Some(_)) => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            (false, Some(_), Some(_)) => {
                errors.push(CompileError::AmbiguousPath { span });
                return err(warnings, errors);
            }
            (false, None, None) => {
                errors.push(CompileError::SymbolNotFound {
                    name: call_path_binding.inner.suffix,
                });
                return err(warnings, errors);
            }
        };

        ok(exp, warnings, errors)
    }

    #[allow(clippy::too_many_arguments)]
    fn type_check_abi_cast(
        mut ctx: TypeCheckContext,
        abi_name: CallPath,
        address: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        // TODO use lib-std's Address type instead of b256
        // type check the address and make sure it is
        let err_span = address.span();
        let address_expr = {
            let ctx = ctx
                .by_ref()
                .with_help_text("An address that is being ABI cast must be of type b256")
                .with_type_annotation(insert_type(TypeInfo::B256));
            check!(
                TypedExpression::type_check(ctx, address),
                error_recovery_expr(err_span),
                warnings,
                errors
            )
        };
        // look up the call path and get the declaration it references
        let abi = check!(
            ctx.namespace.resolve_call_path(&abi_name).cloned(),
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
                            ctx.namespace.resolve_call_path(&abi_name).cloned(),
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
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown))
                .with_mode(Mode::ImplAbiFn);
            type_checked_fn_buf.push(check!(
                TypedFunctionDeclaration::type_check(ctx, method.clone()),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        functions_buf.append(&mut type_checked_fn_buf);
        ctx.namespace
            .insert_trait_implementation(abi_name.clone(), return_type, functions_buf);
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

    fn type_check_array(
        mut ctx: TypeCheckContext,
        contents: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        if contents.is_empty() {
            let unknown_type = insert_type(TypeInfo::Unknown);
            return ok(
                TypedExpression {
                    expression: TypedExpressionVariant::Array {
                        contents: Vec::new(),
                    },
                    return_type: insert_type(TypeInfo::Array(unknown_type, 0, unknown_type)),
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
                let ctx = ctx
                    .by_ref()
                    .with_help_text("")
                    .with_type_annotation(insert_type(TypeInfo::Unknown));
                check!(
                    Self::type_check(ctx, expr),
                    error_recovery_expr(span),
                    warnings,
                    errors
                )
            })
            .collect();

        let elem_type = typed_contents[0].return_type;
        for typed_elem in &typed_contents[1..] {
            let (mut new_warnings, new_errors) = ctx
                .by_ref()
                .with_type_annotation(elem_type)
                .unify_with_self(typed_elem.return_type, &typed_elem.span);
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
                return_type: insert_type(TypeInfo::Array(elem_type, array_count, elem_type)),
                is_constant: IsConstant::No, // Maybe?
                span,
            },
            warnings,
            errors,
        )
    }

    fn type_check_array_index(
        mut ctx: TypeCheckContext,
        prefix: Expression,
        index: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let prefix_te = {
            let ctx = ctx
                .by_ref()
                .with_help_text("")
                .with_type_annotation(insert_type(TypeInfo::Unknown));
            check!(
                TypedExpression::type_check(ctx, prefix.clone()),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        // If the return type is a static array then create a TypedArrayIndex.
        if let TypeInfo::Array(elem_type_id, _, _) = look_up_type_id(prefix_te.return_type) {
            let type_info_u64 = TypeInfo::UnsignedInteger(IntegerBits::SixtyFour);
            let ctx = ctx
                .with_help_text("")
                .with_type_annotation(insert_type(type_info_u64));
            let index_te = check!(
                TypedExpression::type_check(ctx, index),
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
            let method_name = TypeBinding {
                inner: MethodName::FromTrait {
                    call_path: CallPath {
                        prefixes: vec![
                            Ident::new_with_override("core", span.clone()),
                            Ident::new_with_override("ops", span.clone()),
                        ],
                        suffix: Ident::new_with_override("index", span.clone()),
                        is_absolute: true,
                    },
                },
                type_arguments: vec![],
                span: span.clone(),
            };
            type_check_method_application(ctx, method_name, vec![], vec![prefix, index], span)
        }
    }

    fn type_check_intrinsic_function(
        ctx: TypeCheckContext,
        kind_binding: TypeBinding<Intrinsic>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let (intrinsic_function, return_type) = check!(
            TypedIntrinsicFunctionKind::type_check(ctx, kind_binding, arguments, span.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = TypedExpression {
            expression: TypedExpressionVariant::IntrinsicFunction(intrinsic_function),
            return_type,
            is_constant: IsConstant::No,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_while_loop(
        mut ctx: TypeCheckContext,
        condition: Expression,
        body: CodeBlock,
        span: Span,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let typed_condition = {
            let ctx = ctx
                .by_ref()
                .with_type_annotation(insert_type(TypeInfo::Boolean))
                .with_help_text("A while loop's loop condition must be a boolean expression.");
            check!(
                TypedExpression::type_check(ctx, condition),
                return err(warnings, errors),
                warnings,
                errors
            )
        };

        let unit_ty = insert_type(TypeInfo::Tuple(Vec::new()));
        let ctx = ctx.with_type_annotation(unit_ty).with_help_text(
            "A while loop's loop body cannot implicitly return a value. Try \
                 assigning it to a mutable variable declared outside of the loop \
                 instead.",
        );
        let (typed_body, _block_implicit_return) = check!(
            TypedCodeBlock::type_check(ctx, body),
            return err(warnings, errors),
            warnings,
            errors
        );
        let exp = TypedExpression {
            expression: TypedExpressionVariant::WhileLoop {
                condition: Box::new(typed_condition),
                body: typed_body,
            },
            return_type: unit_ty,
            is_constant: IsConstant::Yes,
            span,
        };
        ok(exp, warnings, errors)
    }

    fn type_check_reassignment(
        ctx: TypeCheckContext,
        lhs: ReassignmentTarget,
        rhs: Expression,
        span: Span,
    ) -> CompileResult<Self> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let ctx = ctx
            .with_type_annotation(insert_type(TypeInfo::Unknown))
            .with_help_text("");
        // ensure that the lhs is a variable expression or struct field access
        match lhs {
            ReassignmentTarget::VariableExpression(var) => {
                let mut expr = var;
                let mut names_vec = Vec::new();
                let (base_name, final_return_type) = loop {
                    match expr.kind {
                        ExpressionKind::Variable(name) => {
                            // check that the reassigned name exists
                            let unknown_decl = check!(
                                ctx.namespace.resolve_symbol(&name).cloned(),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let variable_decl = check!(
                                unknown_decl.expect_variable().cloned(),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            if !variable_decl.mutability.is_mutable() {
                                errors.push(CompileError::AssignmentToNonMutable { name });
                                return err(warnings, errors);
                            }
                            break (name, variable_decl.body.return_type);
                        }
                        ExpressionKind::Subfield(SubfieldExpression {
                            prefix,
                            field_to_access,
                            ..
                        }) => {
                            names_vec.push(ProjectionKind::StructField {
                                name: field_to_access,
                            });
                            expr = prefix;
                        }
                        ExpressionKind::TupleIndex(TupleIndexExpression {
                            prefix,
                            index,
                            index_span,
                            ..
                        }) => {
                            names_vec.push(ProjectionKind::TupleField { index, index_span });
                            expr = prefix;
                        }
                        _ => {
                            errors.push(CompileError::InvalidExpressionOnLhs { span });
                            return err(warnings, errors);
                        }
                    }
                };
                let names_vec = names_vec.into_iter().rev().collect::<Vec<_>>();
                let (ty_of_field, _ty_of_parent) = check!(
                    ctx.namespace.find_subfield_type(&base_name, &names_vec),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the reassignment
                let ctx = ctx.with_type_annotation(ty_of_field).with_help_text("");
                let rhs_span = rhs.span();
                let rhs = check!(
                    TypedExpression::type_check(ctx, rhs),
                    error_recovery_expr(rhs_span),
                    warnings,
                    errors
                );

                ok(
                    TypedExpression {
                        expression: TypedExpressionVariant::Reassignment(Box::new(
                            TypedReassignment {
                                lhs_base_name: base_name,
                                lhs_type: final_return_type,
                                lhs_indices: names_vec,
                                rhs,
                            },
                        )),
                        return_type: crate::type_system::insert_type(TypeInfo::Tuple(Vec::new())),
                        // TODO: if the rhs is constant then this should be constant, no?
                        is_constant: IsConstant::No,
                        span,
                    },
                    warnings,
                    errors,
                )
            }
            ReassignmentTarget::StorageField(fields) => {
                let ctx = ctx
                    .with_type_annotation(insert_type(TypeInfo::Unknown))
                    .with_help_text("");
                let reassignment = check!(
                    reassign_storage_subfield(ctx, fields, rhs, span.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors,
                );
                ok(
                    TypedExpression {
                        expression: TypedExpressionVariant::StorageReassignment(Box::new(
                            reassignment,
                        )),
                        return_type: crate::type_system::insert_type(TypeInfo::Tuple(Vec::new())),
                        is_constant: IsConstant::No,
                        span,
                    },
                    warnings,
                    errors,
                )
            }
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn do_type_check(expr: Expression, type_annotation: TypeId) -> CompileResult<TypedExpression> {
        let mut namespace = Namespace::init_root(namespace::Module::default());
        let ctx = TypeCheckContext::from_root(&mut namespace).with_type_annotation(type_annotation);
        TypedExpression::type_check(ctx, expr)
    }

    fn do_type_check_for_boolx2(expr: Expression) -> CompileResult<TypedExpression> {
        do_type_check(
            expr,
            insert_type(TypeInfo::Array(
                insert_type(TypeInfo::Boolean),
                2,
                insert_type(TypeInfo::Boolean),
            )),
        )
    }

    #[test]
    fn test_array_type_check_non_homogeneous_0() {
        // [true, 0] -- first element is correct, assumes type is [bool; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(vec![
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::U64(0)),
                    span: Span::dummy(),
                },
            ]),
            span: Span::dummy(),
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.to_string() == "bool"
                                && received.to_string() == "u64"));
    }

    #[test]
    fn test_array_type_check_non_homogeneous_1() {
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(vec![
                Expression {
                    kind: ExpressionKind::Literal(Literal::U64(0)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
            ]),
            span: Span::dummy(),
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 2);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.to_string() == "u64"
                                && received.to_string() == "bool"));
        assert!(matches!(&comp_res.errors[1],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.to_string() == "[bool; 2]"
                                && received.to_string() == "[u64; 2]"));
    }

    #[test]
    fn test_array_type_check_bad_count() {
        // [0, false] -- first element is incorrect, assumes type is [u64; 2].
        let expr = Expression {
            kind: ExpressionKind::Array(vec![
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
                Expression {
                    kind: ExpressionKind::Literal(Literal::Boolean(true)),
                    span: Span::dummy(),
                },
            ]),
            span: Span::dummy(),
        };

        let comp_res = do_type_check_for_boolx2(expr);
        assert!(comp_res.errors.len() == 1);
        assert!(matches!(&comp_res.errors[0],
                         CompileError::TypeError(TypeError::MismatchedType {
                             expected,
                             received,
                             ..
                         }) if expected.to_string() == "[bool; 2]"
                                && received.to_string() == "[bool; 3]"));
    }

    #[test]
    fn test_array_type_check_empty() {
        let expr = Expression {
            kind: ExpressionKind::Array(Vec::new()),
            span: Span::dummy(),
        };

        let comp_res = do_type_check(
            expr,
            insert_type(TypeInfo::Array(
                insert_type(TypeInfo::Boolean),
                0,
                insert_type(TypeInfo::Boolean),
            )),
        );
        assert!(comp_res.warnings.is_empty() && comp_res.errors.is_empty());
    }
}
fn disallow_opcode(op: &Ident) -> CompileResult<()> {
    let mut errors = vec![];

    match op.as_str().to_lowercase().as_str() {
        "ji" => {
            errors.push(CompileError::DisallowedJi { span: op.span() });
        }
        "jnei" => {
            errors.push(CompileError::DisallowedJnei { span: op.span() });
        }
        "jnzi" => {
            errors.push(CompileError::DisallowedJnzi { span: op.span() });
        }
        _ => (),
    };
    if errors.is_empty() {
        ok((), vec![], vec![])
    } else {
        err(vec![], errors)
    }
}
