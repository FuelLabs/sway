use itertools::Itertools;
use sway_error::{
    error::{CompileError, StructFieldUsageContext},
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Span, Spanned};

use crate::{
    decl_engine::DeclRefStruct,
    language::{
        parsed::*,
        ty::{self, StructAccessInfo, TyStructField},
        CallPath, Visibility,
    },
    semantic_analysis::{
        type_check_context::EnforceTypeArguments, GenericShadowingMode, TypeCheckContext,
    },
    type_system::*,
    Namespace,
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
        .check_absolute_path_to_submodule(handler, &type_info_prefix)?;

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
    let struct_decl = (*decl_engine.get_struct(&struct_ref)).clone();

    let (struct_can_be_changed, is_public_struct_access) =
        StructAccessInfo::get_info(&struct_decl, ctx.namespace).into();
    let struct_has_private_fields = struct_decl.has_private_fields();
    let struct_can_be_instantiated = !is_public_struct_access || !struct_has_private_fields;
    let all_fields_are_private = struct_decl.has_only_private_fields();
    let struct_is_empty = struct_decl.is_empty();
    let struct_name = struct_decl.call_path.suffix;

    let struct_fields = struct_decl.fields;
    let mut struct_fields = struct_fields;

    // To avoid conflicting and overlapping errors, we follow the Rust approach:
    // - Missing fields are reported only if the struct can actually be instantiated.
    // - Individual fields issues are always reported: private field access, non-existing fields.

    let typed_fields = type_check_field_arguments(
        handler,
        ctx.by_ref(),
        &fields,
        &struct_name,
        &mut struct_fields,
        &span,
        &struct_decl.span,
        // Emit the missing fields error only if the struct can actually be instantiated.
        struct_can_be_instantiated,
    )?;

    if !struct_can_be_instantiated {
        let constructors = collect_struct_constructors(
            ctx.namespace,
            ctx.engines,
            type_id,
            ctx.storage_declaration(),
        );

        handler.emit_err(CompileError::StructCannotBeInstantiated {
            struct_name: struct_name.clone(),
            span: inner_span.clone(),
            struct_decl_span: struct_decl.span.clone(),
            private_fields: struct_fields
                .iter()
                .filter(|field| field.is_private())
                .map(|field| field.name.clone())
                .collect(),
            constructors,
            all_fields_are_private,
            is_in_storage_declaration: ctx.storage_declaration(),
            struct_can_be_changed,
        });
    }

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

    // Check that there are no extra fields.
    for field in fields.iter() {
        if !struct_fields.iter().any(|x| x.name == field.name) {
            handler.emit_err(CompileError::StructFieldDoesNotExist {
                field_name: (&field.name).into(), // Explicit borrow to force the `From<&BaseIdent>` instead of `From<BaseIdent>`.
                available_fields: TyStructField::accessible_fields_names(
                    &struct_fields,
                    is_public_struct_access,
                ),
                is_public_struct_access,
                struct_name: struct_name.clone(),
                struct_decl_span: struct_decl.span.clone(),
                struct_is_empty,
                usage_context: if ctx.storage_declaration() {
                    StructFieldUsageContext::StorageDeclaration {
                        struct_can_be_instantiated,
                    }
                } else {
                    StructFieldUsageContext::StructInstantiation {
                        struct_can_be_instantiated,
                    }
                },
            });
        }
    }

    // If the current module being checked is not a submodule of the
    // module in which the struct is declared, check for private fields usage.
    if is_public_struct_access {
        for field in fields {
            if let Some(ty_field) = struct_fields.iter().find(|x| x.name == field.name) {
                if ty_field.is_private() {
                    handler.emit_err(CompileError::StructFieldIsPrivate {
                        field_name: (&field.name).into(),
                        struct_name: struct_name.clone(),
                        field_decl_span: ty_field.name.span(),
                        struct_can_be_changed,
                        usage_context: if ctx.storage_declaration() {
                            StructFieldUsageContext::StorageDeclaration {
                                struct_can_be_instantiated,
                            }
                        } else {
                            StructFieldUsageContext::StructInstantiation {
                                struct_can_be_instantiated,
                            }
                        },
                    });
                }
            }
        }
    }

    let mut struct_namespace = ctx.namespace.clone();
    ctx.with_generic_shadowing_mode(GenericShadowingMode::Allow)
        .scoped(&mut struct_namespace, |mut struct_ctx| {
            // Insert struct type parameter into namespace.
            // This is required so check_type_parameter_bounds can resolve generic trait type parameters.
            for type_parameter in struct_decl.type_parameters {
                type_parameter.insert_into_namespace_self(handler, struct_ctx.by_ref())?;
            }

            type_id.check_type_parameter_bounds(handler, struct_ctx, &span, None)?;

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
        })
}

fn collect_struct_constructors(
    namespace: &Namespace,
    engines: &crate::Engines,
    struct_type_id: TypeId,
    is_in_storage_declaration: bool,
) -> Vec<String> {
    // Searching only for public constructors is a bit too restrictive because we can also have them in local private impls.
    // Checking that would be a questionable additional effort considering that this search gives good suggestions for
    // common patterns in which constructors can be found.
    // Also, strictly speaking, we could also have public module functions that create structs,
    // but that would be a way too much of suggestions, and moreover, it is also not a design pattern/guideline
    // that we wish to encourage.
    namespace
        .module()
        .current_items()
        .get_items_for_type(engines, struct_type_id)
        .iter()
        .filter_map(|item| match item {
            ty::TyTraitItem::Fn(fn_decl_id) => Some(fn_decl_id),
            _ => None,
        })
        .map(|fn_decl_id| engines.de().get_function(fn_decl_id))
        .filter(|fn_decl| {
            matches!(fn_decl.visibility, Visibility::Public)
                    && fn_decl
                        .is_constructor(engines, struct_type_id)
                        .unwrap_or_default()
                    // For suggestions in storage declarations, we go for the simplest heuristics possible -
                    // returning only parameterless constructors. Doing the const evaluation here would be
                    // a questionable additional effort considering that this simple heuristics will give
                    // us all the most common constructors like `default()` or `new()`.
                    && (!is_in_storage_declaration || fn_decl.parameters.is_empty())
        })
        .map(|fn_decl| {
            // Removing the return type from the signature by searching for last `->` will work as long as we don't have something like `Fn`.
            format!("{}", engines.help_out((*fn_decl).clone()))
                .rsplit_once(" -> ")
                .unwrap()
                .0
                .to_string()
        })
        .sorted()
        .dedup()
        .collect_vec()
}

/// Type checks the field arguments.
#[allow(clippy::too_many_arguments)]
fn type_check_field_arguments(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    fields: &[StructExpressionField],
    struct_name: &Ident,
    struct_fields: &mut [ty::TyStructField],
    span: &Span,
    struct_decl_span: &Span,
    emit_missing_fields_error: bool,
) -> Result<Vec<ty::TyStructExpressionField>, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    let mut typed_fields = vec![];
    let mut missing_fields = vec![];

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
                missing_fields.push(struct_field.name.clone());

                let err = Handler::default().emit_err(
                    CompileError::StructInstantiationMissingFieldForErrorRecovery {
                        field_name: struct_field.name.clone(),
                        struct_name: struct_name.clone(),
                        span: span.clone(),
                    },
                );

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

    if emit_missing_fields_error && !missing_fields.is_empty() {
        handler.emit_err(CompileError::StructInstantiationMissingFields {
            field_names: missing_fields,
            struct_name: struct_name.clone(),
            span: span.clone(),
            struct_decl_span: struct_decl_span.clone(),
            total_number_of_fields: struct_fields.len(),
        });
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
