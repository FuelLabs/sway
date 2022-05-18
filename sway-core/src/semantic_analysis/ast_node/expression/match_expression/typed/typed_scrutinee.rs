use sway_types::{Ident, Span};

use crate::semantic_analysis::declaration::{CreateTypeId, EnforceTypeArguments};
use crate::semantic_analysis::namespace::Namespace;
use crate::semantic_analysis::TypedEnumVariant;
use crate::CompileError;
use crate::{
    error::{err, ok},
    type_engine::{insert_type, TypeId},
    CompileResult, Literal, Scrutinee, TypeArgument, TypeInfo,
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
    #[allow(dead_code)]
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
        namespace: &mut Namespace,
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
                // find the struct definition from the name
                let unknown_decl = check!(
                    namespace.resolve_symbol(&struct_name).cloned(),
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
                let struct_decl = check!(
                    namespace.monomorphize(
                        struct_decl,
                        vec!(),
                        EnforceTypeArguments::No,
                        Some(self_type),
                        None
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the fields
                let mut typed_fields = vec![];
                for field in fields.into_iter() {
                    // ensure that the struct definition has this field
                    let _ = check!(
                        struct_decl.expect_field(&field.field),
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
            Scrutinee::EnumScrutinee { span, .. } => {
                errors.push(CompileError::Unimplemented(
                    "matching on enums is unimplemented",
                    span,
                ));
                return err(warnings, errors);
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
