use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::DeclRefStruct,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{type_check_context::EnforceTypeArguments, TypeCheckContext},
    type_system::*,
};

const UNIFY_STRUCT_FIELD_HELP_TEXT: &str =
    "Struct field's type must match the type specified in its declaration.";

pub(crate) fn struct_instantiation(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    mut call_path_binding: TypeBinding<CallPath>,
    fields: Vec<StructExpressionField>,
    span: Span,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    // We need the call_path_binding to have types that point to proper definitions so the LSP can
    // look for them, but its types haven't been resolved yet.
    // To that end we do a dummy type check which has the side effect of resolving the types.
    let _: Result<(DeclRefStruct, _, _), _> =
        TypeBinding::type_check(&mut call_path_binding, &Handler::default(), ctx.by_ref());

    let TypeBinding {
        inner: CallPath {
            prefixes, suffix, ..
        },
        type_arguments,
        span: inner_span,
    } = call_path_binding.clone();

    if let TypeArgs::Prefix(_) = type_arguments {
        return Err(
            handler.emit_err(CompileError::DoesNotTakeTypeArgumentsAsPrefix {
                name: suffix,
                span: type_arguments.span(),
            }),
        );
    }

    let type_arguments = type_arguments.to_vec();

    let type_info = match (suffix.as_str(), type_arguments.is_empty()) {
        ("Self", true) => TypeInfo::new_self_type(suffix.span()),
        ("Self", false) => {
            return Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed {
                span: suffix.span(),
            }));
        }
        (_, true) => TypeInfo::Custom {
            qualified_call_path: suffix.clone().into(),
            type_arguments: None,
            root_type_id: None,
        },
        (_, false) => TypeInfo::Custom {
            qualified_call_path: suffix.clone().into(),
            type_arguments: Some(type_arguments),
            root_type_id: None,
        },
    };

    // find the module that the struct decl is in
    let type_info_prefix = ctx.namespace.find_module_path(&prefixes);
    ctx.namespace
        .root()
        .check_submodule(handler, &type_info_prefix)?;

    // resolve the type of the struct decl
    let type_id = ctx
        .resolve_type(
            handler,
            type_engine.insert(engines, type_info, suffix.span().source_id()),
            &inner_span,
            EnforceTypeArguments::No,
            Some(&type_info_prefix),
        )
        .unwrap_or_else(|err| type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None));

    // extract the struct name and fields from the type info
    let type_info = type_engine.get(type_id);
    let struct_ref = type_info.expect_struct(handler, engines, &span)?;
    let struct_decl = decl_engine.get_struct(&struct_ref);
    let struct_name = struct_decl.call_path.suffix;
    let struct_fields = struct_decl.fields;
    let mut struct_fields = struct_fields;

    let typed_fields = type_check_field_arguments(
        handler,
        ctx.by_ref(),
        &fields,
        &struct_name,
        &mut struct_fields,
        &span,
    )?;

    unify_field_arguments_and_struct_fields(handler, ctx.by_ref(), &typed_fields, &struct_fields)?;

    // Unify type id with type annotation so eventual generic type parameters are properly resolved.
    // When a generic type parameter is not used in field arguments it should be unified with type annotation.
    type_engine.unify(
        handler,
        engines,
        type_id,
        ctx.type_annotation(),
        &span,
        "Struct type must match the type specified in its declaration.",
        None,
    );

    // check that there are no extra fields
    for field in fields {
        if !struct_fields.iter().any(|x| x.name == field.name) {
            handler.emit_err(CompileError::StructDoesNotHaveField {
                field_name: field.name.clone(),
                struct_name: struct_name.clone(),
                span: field.span,
            });
        }
    }

    type_id.check_type_parameter_bounds(handler, ctx, &span, vec![])?;

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

    Ok(exp)
}

/// Type checks the field arguments.
fn type_check_field_arguments(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    fields: &[StructExpressionField],
    struct_name: &Ident,
    struct_fields: &mut [ty::TyStructField],
    span: &Span,
) -> Result<Vec<ty::TyStructExpressionField>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut typed_fields = vec![];

    for struct_field in struct_fields.iter_mut() {
        match fields.iter().find(|x| x.name == struct_field.name) {
            Some(field) => {
                let ctx = ctx
                    .by_ref()
                    .with_help_text(UNIFY_STRUCT_FIELD_HELP_TEXT)
                    .with_type_annotation(struct_field.type_argument.type_id)
                    .with_unify_generic(true);
                let value = match ty::TyExpression::type_check(handler, ctx, field.value.clone()) {
                    Ok(res) => res,
                    Err(_) => continue,
                };
                typed_fields.push(ty::TyStructExpressionField {
                    value,
                    name: field.name.clone(),
                });
                struct_field.span = field.value.span.clone();
            }
            None => {
                let err = handler.emit_err(CompileError::StructMissingField {
                    field_name: struct_field.name.clone(),
                    struct_name: struct_name.clone(),
                    span: span.clone(),
                });
                typed_fields.push(ty::TyStructExpressionField {
                    name: struct_field.name.clone(),
                    value: ty::TyExpression {
                        expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                        return_type: type_engine.insert(
                            engines,
                            TypeInfo::ErrorRecovery(err),
                            None,
                        ),
                        span: span.clone(),
                    },
                });
            }
        }
    }

    Ok(typed_fields)
}

/// Unifies the field arguments and the types of the fields from the struct
/// definition.
fn unify_field_arguments_and_struct_fields(
    handler: &Handler,
    ctx: TypeCheckContext,
    typed_fields: &[ty::TyStructExpressionField],
    struct_fields: &[ty::TyStructField],
) -> Result<(), ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    handler.scope(|handler| {
        for struct_field in struct_fields.iter() {
            if let Some(typed_field) = typed_fields.iter().find(|x| x.name == struct_field.name) {
                type_engine.unify_with_generic(
                    handler,
                    engines,
                    typed_field.value.return_type,
                    struct_field.type_argument.type_id,
                    &typed_field.value.span,
                    UNIFY_STRUCT_FIELD_HELP_TEXT,
                    None,
                );
            }
        }
        Ok(())
    })
}
