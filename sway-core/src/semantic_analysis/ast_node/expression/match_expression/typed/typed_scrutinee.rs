use sway_error::error::CompileError;
use sway_types::{BaseIdent, Ident, Span, Spanned};

use crate::{
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::TypeCheckContext,
    type_system::*,
};

impl ty::TyScrutinee {
    pub(crate) fn type_check(ctx: TypeCheckContext, scrutinee: Scrutinee) -> CompileResult<Self> {
        let warnings = vec![];
        let errors = vec![];
        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        match scrutinee {
            Scrutinee::CatchAll { span } => {
                let type_id = type_engine.insert(decl_engine, TypeInfo::Unknown);
                let dummy_type_param = TypeParameter {
                    type_id,
                    initial_type_id: type_id,
                    name_ident: BaseIdent::new_with_override("_", span.clone()),
                    trait_constraints: vec![],
                    trait_constraints_span: Span::dummy(),
                };
                let typed_scrutinee = ty::TyScrutinee {
                    variant: ty::TyScrutineeVariant::CatchAll,
                    type_id: type_engine
                        .insert(decl_engine, TypeInfo::Placeholder(dummy_type_param)),
                    span,
                };
                ok(typed_scrutinee, warnings, errors)
            }
            Scrutinee::Literal { value, span } => {
                let typed_scrutinee = ty::TyScrutinee {
                    variant: ty::TyScrutineeVariant::Literal(value.clone()),
                    type_id: type_engine.insert(decl_engine, value.to_typeinfo()),
                    span,
                };
                ok(typed_scrutinee, warnings, errors)
            }
            Scrutinee::Variable { name, span } => type_check_variable(ctx, name, span),
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                span,
            } => type_check_struct(ctx, struct_name.suffix, fields, span),
            Scrutinee::EnumScrutinee {
                call_path,
                value,
                span,
            } => type_check_enum(ctx, call_path, *value, span),
            Scrutinee::Tuple { elems, span } => type_check_tuple(ctx, elems, span),
            Scrutinee::Error { .. } => err(vec![], vec![]),
        }
    }
}

fn type_check_variable(
    ctx: TypeCheckContext,
    name: Ident,
    span: Span,
) -> CompileResult<ty::TyScrutinee> {
    let warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;

    let typed_scrutinee = match ctx.namespace.resolve_symbol(&name).value {
        // If this variable is a constant, then we turn it into a [TyScrutinee::Constant](ty::TyScrutinee::Constant).
        Some(ty::TyDeclaration::ConstantDeclaration { decl_id, .. }) => {
            let constant_decl = decl_engine.get_constant(decl_id);
            let value = match constant_decl.value {
                Some(ref value) => value,
                None => {
                    errors.push(CompileError::Internal(
                        "constant value does not contain expression",
                        span,
                    ));
                    return err(warnings, errors);
                }
            };
            let literal = match value.extract_literal_value() {
                Some(value) => value,
                None => {
                    errors.push(CompileError::Unimplemented(
                        "constant values of this type are not supported yet",
                        span,
                    ));
                    return err(warnings, errors);
                }
            };
            ty::TyScrutinee {
                type_id: value.return_type,
                variant: ty::TyScrutineeVariant::Constant(name, literal, constant_decl),
                span,
            }
        }
        // Variable isn't a constant, so so we turn it into a [ty::TyScrutinee::Variable].
        _ => ty::TyScrutinee {
            variant: ty::TyScrutineeVariant::Variable(name),
            type_id: type_engine.insert(decl_engine, TypeInfo::Unknown),
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
) -> CompileResult<ty::TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let decl_engine = ctx.decl_engine;

    // find the struct definition from the name
    let unknown_decl = check!(
        ctx.namespace.resolve_symbol(&struct_name).cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut struct_decl = check!(
        unknown_decl.expect_struct(decl_engine),
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
                let struct_field = check!(
                    struct_decl.expect_field(&field),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                // type check the nested scrutinee
                let typed_scrutinee = match scrutinee {
                    None => None,
                    Some(scrutinee) => Some(check!(
                        ty::TyScrutinee::type_check(ctx.by_ref(), scrutinee),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )),
                };
                typed_fields.push(ty::TyStructScrutineeField {
                    field,
                    scrutinee: typed_scrutinee,
                    span,
                    field_def_name: struct_field.name.clone(),
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

    let typed_scrutinee = ty::TyScrutinee {
        type_id: struct_decl.create_type_id(ctx.engines()),
        span,
        variant: ty::TyScrutineeVariant::StructScrutinee {
            struct_name,
            decl_name: struct_decl.call_path.suffix,
            fields: typed_fields,
        },
    };

    ok(typed_scrutinee, warnings, errors)
}

fn type_check_enum(
    mut ctx: TypeCheckContext,
    call_path: CallPath<Ident>,
    value: Scrutinee,
    span: Span,
) -> CompileResult<ty::TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let decl_engine = ctx.decl_engine;

    let mut prefixes = call_path.prefixes.clone();
    let enum_callpath = match prefixes.pop() {
        Some(enum_name) => CallPath {
            suffix: enum_name,
            prefixes,
            is_absolute: call_path.is_absolute,
        },
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
        ctx.namespace.resolve_call_path(&enum_callpath).cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut enum_decl = check!(
        unknown_decl.expect_enum(decl_engine),
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
            &enum_callpath.span()
        ),
        return err(warnings, errors),
        warnings,
        errors
    );
    let enum_type_id = enum_decl.create_type_id(ctx.engines());

    // check to see if the variant exists and grab it if it does
    let variant = check!(
        enum_decl.expect_variant_from_name(&variant_name).cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );

    // type check the nested scrutinee
    let typed_value = check!(
        ty::TyScrutinee::type_check(ctx, value),
        return err(warnings, errors),
        warnings,
        errors
    );

    let typed_scrutinee = ty::TyScrutinee {
        variant: ty::TyScrutineeVariant::EnumScrutinee {
            call_path,
            decl_name: enum_decl.call_path.suffix,
            variant: Box::new(variant),
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
) -> CompileResult<ty::TyScrutinee> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;

    let mut typed_elems = vec![];
    for elem in elems.into_iter() {
        typed_elems.push(check!(
            ty::TyScrutinee::type_check(ctx.by_ref(), elem),
            continue,
            warnings,
            errors
        ));
    }
    let type_id = type_engine.insert(
        decl_engine,
        TypeInfo::Tuple(
            typed_elems
                .iter()
                .map(|x| TypeArgument {
                    type_id: x.type_id,
                    initial_type_id: x.type_id,
                    span: span.clone(),
                    call_path_tree: None,
                })
                .collect(),
        ),
    );
    let typed_scrutinee = ty::TyScrutinee {
        variant: ty::TyScrutineeVariant::Tuple(typed_elems),
        type_id,
        span,
    };

    ok(typed_scrutinee, warnings, errors)
}
