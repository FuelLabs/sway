use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::{TypeCheckContext, TypedEnumVariant},
    type_system::{insert_type, CreateTypeId, EnforceTypeArguments, TypeArgument, TypeId},
    CompileError, CompileResult, Literal, Scrutinee, StructScrutineeField, TypeInfo,
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
        mut ctx: TypeCheckContext,
        scrutinee: Scrutinee,
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
                    ctx.namespace.resolve_symbol(&struct_name).cloned(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut struct_decl = check!(
                    unknown_decl.expect_struct(&span),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // monomorphize the struct definition
                check!(
                    ctx.monomorphize(
                        &mut struct_decl,
                        &mut [],
                        EnforceTypeArguments::No,
                        &struct_name.span()
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the fields
                let mut typed_fields = vec![];
                let mut rest_pattern = None;
                for field in fields.into_iter() {
                    match field {
                        StructScrutineeField::Rest { .. } => rest_pattern = Some(field),
                        StructScrutineeField::Field {
                            field,
                            scrutinee,
                            span,
                        } => {
                            // ensure that the struct definition has this field
                            let _ = check!(
                                struct_decl.expect_field(&field),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            // type check the nested scrutinee
                            let typed_scrutinee = match scrutinee {
                                None => None,
                                Some(scrutinee) => Some(check!(
                                    TypedScrutinee::type_check(ctx.by_ref(), scrutinee),
                                    return err(warnings, errors),
                                    warnings,
                                    errors
                                )),
                            };
                            typed_fields.push(TypedStructScrutineeField {
                                field,
                                scrutinee: typed_scrutinee,
                                span,
                            });
                        }
                    }
                }
                // ensure that the pattern uses all fields of the struct unless the rest pattern is present
                if (struct_decl.fields.len() != typed_fields.len()) && rest_pattern.is_none() {
                    let missing_fields = struct_decl
                        .fields
                        .iter()
                        .filter(|f| !typed_fields.iter().any(|tf| f.name == tf.field))
                        .map(|f| f.name.to_string())
                        .collect::<Vec<_>>();

                    errors.push(CompileError::MatchStructPatternMissingFields {
                        span,
                        missing_fields,
                    });

                    return err(warnings, errors);
                }
                TypedScrutinee {
                    variant: TypedScrutineeVariant::StructScrutinee(typed_fields),
                    type_id: struct_decl.create_type_id(),
                    span,
                }
            }
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span,
            } => {
                let enum_name = match call_path.prefixes.last() {
                    Some(enum_name) => enum_name,
                    None => {
                        errors.push(CompileError::EnumNotFound {
                            name: call_path.suffix.clone(),
                            span: call_path.suffix.span(),
                        });
                        return err(warnings, errors);
                    }
                };
                let variant_name = call_path.suffix;
                // find the enum definition from the name
                let unknown_decl = check!(
                    ctx.namespace.resolve_symbol(enum_name).cloned(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let mut enum_decl = check!(
                    unknown_decl.expect_enum(&enum_name.span()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // monomorphize the enum definition
                check!(
                    ctx.monomorphize(
                        &mut enum_decl,
                        &mut [],
                        EnforceTypeArguments::No,
                        &enum_name.span()
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let enum_type_id = enum_decl.create_type_id();
                // check to see if the variant exists and grab it if it does
                let variant = check!(
                    enum_decl.expect_variant_from_name(&variant_name).cloned(),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the nested scrutinee
                let typed_value = check!(
                    TypedScrutinee::type_check(ctx, *value),
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
                        TypedScrutinee::type_check(ctx.by_ref(), elem),
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
                                initial_type_id: x.type_id,
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
