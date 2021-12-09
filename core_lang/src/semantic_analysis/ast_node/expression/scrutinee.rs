use std::collections::{HashMap, HashSet};

use crate::{
    control_flow_analysis::ControlFlowGraph,
    error::ok,
    semantic_analysis::ast_node::{IsConstant, TypedVariableDeclaration},
    type_engine::{insert_type, look_up_type_id, IntegerBits, TypeId},
    BuildConfig, CompileError, CompileResult, CompileWarning, Ident, Literal, Namespace, Scrutinee,
    Span, StructScrutineeField, TypeInfo, TypedDeclaration,
};

use super::{TypedExpression, TypedExpressionVariant, TypedScrutineeVariant};

#[derive(Debug, Clone)]
pub(crate) struct TypedScrutinee<'sc> {
    pub(crate) scrutinee: TypedScrutineeVariant<'sc>,
    pub(crate) return_type: TypeId,
    pub(crate) span: Span<'sc>,
}

impl<'sc> TypedScrutinee<'sc> {
    pub(crate) fn type_check(
        other: Scrutinee<'sc>,
        namespace: &mut Namespace<'sc>,
        primary_expression_type: TypeId,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let scrutinee_span = other.span();

        let res = match other {
            Scrutinee::Unit { span } => Self::type_check_unit(span),
            Scrutinee::Literal { value, span } => Self::type_check_literal(value, span),
            Scrutinee::Variable { name, span } => {
                Self::type_check_variable(name, span, namespace, primary_expression_type)
            }
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span,
            } => Self::type_check_struct(
                struct_name,
                fields,
                span,
                namespace,
                primary_expression_type,
                help_text,
                self_type,
                build_config,
                dead_code_graph,
                dependency_graph,
            ),
        };
        let mut typed_scrutinee = match res.value {
            Some(r) => r,
            None => return res,
        };

        match crate::type_engine::unify_with_self(
            typed_scrutinee.return_type,
            primary_expression_type,
            self_type,
            &scrutinee_span,
        ) {
            Ok(ws) => {
                for warning in ws {
                    warnings.push(CompileWarning {
                        warning_content: warning,
                        span: scrutinee_span.clone(),
                    });
                }
            }
            Err(e) => {
                errors.push(CompileError::TypeError(e));
            }
        };

        typed_scrutinee.return_type = namespace
            .resolve_type_with_self(look_up_type_id(typed_scrutinee.return_type), self_type)
            .unwrap_or_else(|_| {
                errors.push(CompileError::UnknownType {
                    span: scrutinee_span,
                });
                insert_type(TypeInfo::ErrorRecovery)
            });

        ok(typed_scrutinee, warnings, errors)
    }

    pub(crate) fn type_check_unit(span: Span<'sc>) -> CompileResult<'sc, Self> {
        let scrutinee = TypedScrutinee {
            scrutinee: TypedScrutineeVariant::Unit { span: span.clone() },
            return_type: crate::type_engine::insert_type(TypeInfo::Unit),
            span: span.clone(),
        };
        ok(scrutinee, vec![], vec![])
    }

    pub(crate) fn type_check_literal(
        value: Literal<'sc>,
        span: Span<'sc>,
    ) -> CompileResult<'sc, Self> {
        let return_type = match value {
            Literal::String(s) => TypeInfo::Str(s.len() as u64),
            Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
            Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),

            Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
            Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
            Literal::Boolean(_) => TypeInfo::Boolean,
            Literal::Byte(_) => TypeInfo::Byte,
            Literal::B256(_) => TypeInfo::B256,
        };
        let id = crate::type_engine::insert_type(return_type);
        let scrutinee = TypedScrutinee {
            scrutinee: TypedScrutineeVariant::Literal {
                value,
                span: span.clone(),
            },
            return_type: id,
            span,
        };
        ok(scrutinee, vec![], vec![])
    }

    pub(crate) fn type_check_variable(
        name: Ident<'sc>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        primary_expression_type: TypeId,
    ) -> CompileResult<'sc, Self> {
        namespace.insert(
            name.clone(),
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                name: name.clone(),
                body: TypedExpression {
                    expression: TypedExpressionVariant::ScrutineeParameter,
                    return_type: primary_expression_type,
                    is_constant: IsConstant::No,
                    span: name.span.clone(),
                },
                is_mutable: false, // TODO allow mutable function params?
                type_ascription: primary_expression_type,
            }),
        );
        let scrutinee = TypedScrutinee {
            scrutinee: TypedScrutineeVariant::Variable {
                name,
                span: span.clone(),
            },
            return_type: primary_expression_type,
            span,
        };
        ok(scrutinee, vec![], vec![])
    }

    pub(crate) fn type_check_struct(
        struct_name: Ident<'sc>,
        fields: Vec<StructScrutineeField<'sc>>,
        span: Span<'sc>,
        namespace: &mut Namespace<'sc>,
        primary_expression_type: TypeId,
        help_text: impl Into<String> + Clone,
        self_type: TypeId,
        build_config: &BuildConfig,
        dead_code_graph: &mut ControlFlowGraph<'sc>,
        dependency_graph: &mut HashMap<String, HashSet<String>>,
    ) -> CompileResult<'sc, Self> {
        unimplemented!()
    }
}
