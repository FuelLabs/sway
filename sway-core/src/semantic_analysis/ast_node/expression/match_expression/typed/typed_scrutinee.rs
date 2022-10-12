use sway_types::{Ident, Span, Spanned};

use crate::{
    declaration_engine::de_get_constant,
    error::{err, ok},
    language::{parsed::*, ty, CallPath, Literal},
    semantic_analysis::{TyEnumVariant, TypeCheckContext},
    type_system::{insert_type, CreateTypeId, EnforceTypeArguments, TypeArgument, TypeId},
    CompileError, CompileResult, TypeInfo,
};

#[derive(Debug, Clone)]
pub(crate) struct TyScrutinee {
    pub(crate) variant: TyScrutineeVariant,
    pub(crate) type_id: TypeId,
    pub(crate) span: Span,
}

#[derive(Debug, Clone)]
pub(crate) enum TyScrutineeVariant {
    CatchAll,
    Literal(Literal),
    Variable(Ident),
    Constant(Ident, Literal, TypeId),
    StructScrutinee(Ident, Vec<TyStructScrutineeField>),
    #[allow(dead_code)]
    EnumScrutinee {
        call_path: CallPath,
        variant: TyEnumVariant,
        value: Box<TyScrutinee>,
    },
    Tuple(Vec<TyScrutinee>),
}

#[derive(Debug, Clone)]
pub(crate) struct TyStructScrutineeField {
    pub(crate) field: Ident,
    pub(crate) scrutinee: Option<TyScrutinee>,
    pub(crate) span: Span,
}

impl TyScrutinee {
    pub(crate) fn type_check(ctx: TypeCheckContext, scrutinee: Scrutinee) -> CompileResult<Self> {
        let warnings = vec![];
        let errors = vec![];
        match scrutinee {
            Scrutinee::CatchAll { span } => {
                let typed_scrutinee = TyScrutinee {
                    variant: TyScrutineeVariant::CatchAll,
                    type_id: insert_type(TypeInfo::Unknown),
                    span,
                };
                ok(typed_scrutinee, warnings, errors)
            }
            Scrutinee::Literal { value, span } => {
                let typed_scrutinee = TyScrutinee {
                    variant: TyScrutineeVariant::Literal(value.clone()),
                    type_id: insert_type(value.to_typeinfo()),
                    span,
                };
                ok(typed_scrutinee, warnings, errors)
            }
            Scrutinee::Variable { name, span } => type_check_variable(ctx, name, span),
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span,
            } => type_check_struct(ctx, struct_name, fields, span),
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span,
            } => type_check_enum(ctx, call_path, *value, span),
            Scrutinee::Tuple { elems, span } => type_check_tuple(ctx, elems, span),
        }
    }
}

fn type_check_variable(
    ctx: TypeCheckContext,
    name: Ident,
    span: Span,
) -> CompileResult<TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let typed_scrutinee = match ctx.namespace.resolve_symbol(&name).value {
        // If this variable is a constant, then we turn it into a [TyScrutinee::Constant].
        Some(ty::TyDeclaration::ConstantDeclaration(decl_id)) => {
            let constant_decl = check!(
                CompileResult::from(de_get_constant(decl_id.clone(), &span)),
                return err(warnings, errors),
                warnings,
                errors
            );
            let value = match constant_decl.value.extract_literal_value() {
                Some(value) => value,
                None => {
                    errors.push(CompileError::Unimplemented(
                        "constant values of this type are not supported yet",
                        span,
                    ));
                    return err(warnings, errors);
                }
            };
            TyScrutinee {
                variant: TyScrutineeVariant::Constant(name, value, constant_decl.value.return_type),
                type_id: constant_decl.value.return_type,
                span,
            }
        }
        // Variable isn't a constant, so so we turn it into a [TyScrutinee::Variable].
        _ => TyScrutinee {
            variant: TyScrutineeVariant::Variable(name),
            type_id: insert_type(TypeInfo::Unknown),
            span,
        },
    };

    ok(typed_scrutinee, warnings, errors)
}

fn type_check_struct(
    mut ctx: TypeCheckContext,
    struct_name: Ident,
    fields: Vec<StructScrutineeField>,
    span: Span,
) -> CompileResult<TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

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
                        TyScrutinee::type_check(ctx.by_ref(), scrutinee),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )),
                };
                typed_fields.push(TyStructScrutineeField {
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

    let typed_scrutinee = TyScrutinee {
        variant: TyScrutineeVariant::StructScrutinee(struct_decl.name.clone(), typed_fields),
        type_id: struct_decl.create_type_id(),
        span,
    };

    ok(typed_scrutinee, warnings, errors)
}

fn type_check_enum(
    ctx: TypeCheckContext,
    call_path: CallPath<Ident>,
    value: Scrutinee,
    span: Span,
) -> CompileResult<TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

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
    let variant_name = call_path.suffix.clone();

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
        TyScrutinee::type_check(ctx, value),
        return err(warnings, errors),
        warnings,
        errors
    );

    let typed_scrutinee = TyScrutinee {
        variant: TyScrutineeVariant::EnumScrutinee {
            call_path,
            variant,
            value: Box::new(typed_value),
        },
        type_id: enum_type_id,
        span,
    };

    ok(typed_scrutinee, warnings, errors)
}

fn type_check_tuple(
    mut ctx: TypeCheckContext,
    elems: Vec<Scrutinee>,
    span: Span,
) -> CompileResult<TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let mut typed_elems = vec![];
    for elem in elems.into_iter() {
        typed_elems.push(check!(
            TyScrutinee::type_check(ctx.by_ref(), elem),
            continue,
            warnings,
            errors
        ));
    }
    let type_id = insert_type(TypeInfo::Tuple(
        typed_elems
            .iter()
            .map(|x| TypeArgument {
                type_id: x.type_id,
                initial_type_id: x.type_id,
                span: span.clone(),
            })
            .collect(),
    ));
    let typed_scrutinee = TyScrutinee {
        variant: TyScrutineeVariant::Tuple(typed_elems),
        type_id,
        span,
    };

    ok(typed_scrutinee, warnings, errors)
}
