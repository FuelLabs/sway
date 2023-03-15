use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::DeclRefStruct,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::TypeCheckContext,
    type_system::*,
};

pub(crate) fn struct_instantiation(
    mut ctx: TypeCheckContext,
    mut call_path_binding: TypeBinding<CallPath>,
    fields: Vec<StructExpressionField>,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    // We need the call_path_binding to have types that point to proper definitions so the LSP can
    // look for them, but its types haven't been resolved yet.
    // To that end we do a dummy type check which has the side effect of resolving the types.
    let _: CompileResult<(DeclRefStruct, _)> =
        TypeBinding::type_check(&mut call_path_binding, ctx.by_ref());

    let TypeBinding {
        inner: CallPath {
            prefixes, suffix, ..
        },
        type_arguments,
        span: inner_span,
    } = call_path_binding.clone();

    if let TypeArgs::Prefix(_) = type_arguments {
        errors.push(CompileError::DoesNotTakeTypeArgumentsAsPrefix {
            name: suffix,
            span: type_arguments.span(),
        });
        return err(warnings, errors);
    }

    let type_arguments = type_arguments.to_vec();

    let type_info = match (suffix.as_str(), type_arguments.is_empty()) {
        ("Self", true) => TypeInfo::SelfType,
        ("Self", false) => {
            errors.push(CompileError::TypeArgumentsNotAllowed {
                span: suffix.span(),
            });
            return err(warnings, errors);
        }
        (_, true) => TypeInfo::Custom {
            call_path: suffix.into(),
            type_arguments: None,
        },
        (_, false) => TypeInfo::Custom {
            call_path: suffix.into(),
            type_arguments: Some(type_arguments),
        },
    };

    // find the module that the struct decl is in
    let type_info_prefix = ctx.namespace.find_module_path(&prefixes);
    check!(
        ctx.namespace.root().check_submodule(&type_info_prefix),
        return err(warnings, errors),
        warnings,
        errors
    );

    // resolve the type of the struct decl
    let type_id = check!(
        ctx.resolve_type(
            type_engine.insert(decl_engine, type_info),
            &inner_span,
            EnforceTypeArguments::No,
            Some(&type_info_prefix)
        ),
        type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
        warnings,
        errors
    );

    // extract the struct name and fields from the type info
    let type_info = type_engine.get(type_id);
    let struct_ref = check!(
        type_info.expect_struct(engines, &span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let struct_decl = decl_engine.get_struct(&struct_ref);
    let struct_name = struct_decl.call_path.suffix;
    let struct_fields = struct_decl.fields;
    let mut struct_fields = struct_fields;

    let typed_fields = check!(
        type_check_field_arguments(
            ctx.by_ref(),
            &fields,
            &struct_name,
            &mut struct_fields,
            &span
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    check!(
        unify_field_arguments_and_struct_fields(ctx.by_ref(), &typed_fields, &struct_fields,),
        return err(warnings, errors),
        warnings,
        errors
    );

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

    let exp = ty::TyExpression {
        expression: ty::TyExpressionVariant::StructExpression {
            struct_ref,
            fields: typed_fields,
            instantiation_span: inner_span,
            call_path_binding,
        },
        return_type: type_id,
        span,
    };

    ok(exp, warnings, errors)
}

/// Type checks the field arguments.
fn type_check_field_arguments(
    mut ctx: TypeCheckContext,
    fields: &[StructExpressionField],
    struct_name: &Ident,
    struct_fields: &mut [ty::TyStructField],
    span: &Span,
) -> CompileResult<Vec<ty::TyStructExpressionField>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;

    let mut typed_fields = vec![];

    for struct_field in struct_fields.iter_mut() {
        match fields.iter().find(|x| x.name == struct_field.name) {
            Some(field) => {
                let ctx = ctx
                    .by_ref()
                    .with_help_text("")
                    .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));
                let value = check!(
                    ty::TyExpression::type_check(ctx, field.value.clone()),
                    continue,
                    warnings,
                    errors
                );
                typed_fields.push(ty::TyStructExpressionField {
                    value,
                    name: field.name.clone(),
                });
                struct_field.span = field.value.span.clone();
            }
            None => {
                errors.push(CompileError::StructMissingField {
                    field_name: struct_field.name.clone(),
                    struct_name: struct_name.clone(),
                    span: span.clone(),
                });
                typed_fields.push(ty::TyStructExpressionField {
                    name: struct_field.name.clone(),
                    value: ty::TyExpression {
                        expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                        return_type: type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                        span: span.clone(),
                    },
                });
            }
        }
    }

    ok(typed_fields, warnings, errors)
}

/// Unifies the field arguments and the types of the fields from the struct
/// definition.
fn unify_field_arguments_and_struct_fields(
    ctx: TypeCheckContext,
    typed_fields: &[ty::TyStructExpressionField],
    struct_fields: &[ty::TyStructField],
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;

    for struct_field in struct_fields.iter() {
        if let Some(typed_field) = typed_fields.iter().find(|x| x.name == struct_field.name) {
            check!(
                CompileResult::from(type_engine.unify(
                    decl_engine,
                    typed_field.value.return_type,
                    struct_field.type_argument.type_id,
                    &typed_field.value.span,
                    "Struct field's type must match the type specified in its declaration.",
                    None,
                )),
                continue,
                warnings,
                errors
            );
        }
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
