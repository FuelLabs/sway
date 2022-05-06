use sway_types::{Ident, Span};

use crate::semantic_analysis::ast_node::declaration::CreateTypeId;
use crate::semantic_analysis::ast_node::declaration::Monomorphize;
use crate::semantic_analysis::TypedEnumVariant;
use crate::{
    error::{err, ok},
    semantic_analysis::TypedStructField,
    type_engine::{insert_type, TypeId},
    CompileResult, Literal, NamespaceRef, NamespaceWrapper, Scrutinee, TypeArgument, TypeInfo,
};

#[derive(Debug, Clone)]
pub(crate) struct TypedScrutinee {
    pub(crate) variant: TypedScrutineeVariant,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum TypedScrutineeVariant {
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    StructScrutinee(Vec<TypedStructScrutineeField>),
    EnumScrutinee {
        variant: TypedEnumVariant,
        value: Box<TypedScrutinee>,
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
            Scrutinee::CatchAll { span } => TypedScrutinee {
                variant: TypedScrutineeVariant::CatchAll,
                type_id: insert_type(TypeInfo::Unknown),
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
                // grab the struct definition
                let struct_decl = check!(
                    namespace.expect_struct_decl_from_name(&struct_name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // monomorphize the struct definition
                let struct_decl = check!(
                    struct_decl.monomorphize(vec!(), false, &namespace, Some(self_type), None),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the fields
                let mut typed_fields = vec![];
                for field in fields.into_iter() {
                    // ensure that the struct definition has this field
                    let _ = check!(
                        TypedStructField::expect_field_from_fields(
                            &struct_decl.name,
                            &struct_decl.fields,
                            &field.field
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    // type check the nested scrutinee
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
                    variant: TypedScrutineeVariant::StructScrutinee(typed_fields),
                    type_id: struct_decl.type_id(),
                    span,
                }
            }
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span,
            } => {
                // grab the enum name and variant name from the call path
                let enum_name = call_path.prefixes.last().unwrap().clone();
                let variant_name = call_path.suffix;
                // grab the enum definition
                let enum_decl = check!(
                    namespace.expect_enum_decl_from_name(&enum_name), // TODO: edit this so that it is found via call_path
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // monomorphize the enum definition
                let enum_decl = check!(
                    enum_decl.monomorphize(vec!(), false, &namespace, Some(self_type), None),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let enum_type_id = enum_decl.type_id();
                // check to see if the variant exists and grab it if it does
                let variant = check!(
                    enum_decl.expect_variant_from_name(&variant_name),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the nested scrutinee
                let typed_value = check!(
                    TypedScrutinee::type_check(*value, namespace, self_type),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                TypedScrutinee {
                    variant: TypedScrutineeVariant::EnumScrutinee {
                        variant,
                        value: Box::new(typed_value),
                    },
                    type_id: enum_type_id,
                    span,
                }
            }
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
