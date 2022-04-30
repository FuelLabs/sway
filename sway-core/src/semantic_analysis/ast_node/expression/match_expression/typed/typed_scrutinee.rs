use sway_types::{Ident, Span};

use crate::{
    error::{err, ok},
    semantic_analysis::ast_node::expression::typed_expression::monomorphize_with_type_arguments,
    type_engine::{insert_type, TypeId},
    CallPath, CompileError, CompileResult, Literal, NamespaceRef, NamespaceWrapper, Scrutinee,
    TypeArgument, TypeInfo, TypedDeclaration,
};

#[derive(Debug, Clone)]
pub(crate) struct TypedScrutinee {
    pub(crate) variant: TypedScrutineeVariant,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum TypedScrutineeVariant {
    Literal(Literal),
    Variable(Ident),
    StructScrutinee {
        struct_name: Ident,
        fields: Vec<TypedStructScrutineeField>,
    },
    EnumScrutinee {
        call_path: CallPath,
        variable_to_assign: Ident,
    },
    Tuple(Vec<TypedScrutinee>),
}

#[derive(Debug, Clone)]
pub(crate) struct TypedStructScrutineeField {
    pub(crate) field: Ident,
    pub(crate) scrutinee: Option<TypedScrutinee>,
    pub(crate) span: Span,
}

impl TypedScrutinee {
    pub(crate) fn type_check(
        scrutinee: Scrutinee,
        namespace: NamespaceRef,
        self_type: TypeId,
    ) -> CompileResult<Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let typed_scrutinee = match scrutinee {
            Scrutinee::Unit { span } => TypedScrutinee {
                variant: TypedScrutineeVariant::Tuple(vec![]),
                type_id: insert_type(TypeInfo::Tuple(vec![])),
                span,
            },
            Scrutinee::Literal { value, span } => TypedScrutinee {
                variant: TypedScrutineeVariant::Literal(value.clone()),
                type_id: insert_type(value.to_typeinfo()),
                span,
            },
            Scrutinee::Variable { name, span } => TypedScrutinee {
                variant: TypedScrutineeVariant::Variable(name),
                type_id: insert_type(TypeInfo::Unknown),
                span,
            },
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span,
            } => {
                let struct_decl = match namespace.get_symbol(&struct_name).value {
                    Some(TypedDeclaration::StructDeclaration(decl)) => {
                        check!(
                            monomorphize_with_type_arguments(
                                CallPath::from(struct_name.clone()),
                                decl,
                                vec!(),
                                namespace,
                                self_type
                            ),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )
                    }
                    _ => {
                        errors.push(CompileError::StructNotFound {
                            name: struct_name.clone(),
                            span,
                        });
                        return err(warnings, errors);
                    }
                };
                let mut typed_fields = vec![];
                for field in fields.into_iter() {
                    let typed_scrutinee = match field.scrutinee {
                        None => None,
                        Some(scrutinee) => Some(check!(
                            TypedScrutinee::type_check(scrutinee, namespace, self_type),
                            return err(warnings, errors),
                            warnings,
                            errors
                        )),
                    };
                    typed_fields.push(TypedStructScrutineeField {
                        field: field.field,
                        scrutinee: typed_scrutinee,
                        span: field.span,
                    });
                }
                TypedScrutinee {
                    variant: TypedScrutineeVariant::StructScrutinee {
                        struct_name,
                        fields: typed_fields,
                    },
                    type_id: struct_decl.type_id(),
                    span,
                }
            }
            Scrutinee::EnumScrutinee {
                call_path,
                variable_to_assign,
                span,
            } => TypedScrutinee {
                variant: TypedScrutineeVariant::EnumScrutinee {
                    call_path,
                    variable_to_assign,
                },
                type_id: insert_type(TypeInfo::Unknown),
                span,
            },
            Scrutinee::Tuple { elems, span } => {
                let mut typed_elems = vec![];
                for elem in elems.into_iter() {
                    typed_elems.push(check!(
                        TypedScrutinee::type_check(elem, namespace, self_type),
                        continue,
                        warnings,
                        errors
                    ));
                }
                TypedScrutinee {
                    variant: TypedScrutineeVariant::Tuple(typed_elems.clone()),
                    type_id: insert_type(TypeInfo::Tuple(
                        typed_elems
                            .into_iter()
                            .map(|x| TypeArgument {
                                type_id: x.type_id,
                                span: span.clone(),
                            })
                            .collect(),
                    )),
                    span,
                }
            }
        };
        ok(typed_scrutinee, warnings, errors)
    }
}
