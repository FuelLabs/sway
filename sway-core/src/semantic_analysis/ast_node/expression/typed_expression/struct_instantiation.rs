use std::collections::BTreeSet;

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
    namespace::ResolvedTraitImplItem,
    semantic_analysis::{GenericShadowingMode, TypeCheckContext},
    type_system::*,
    Engines, Namespace,
};

const UNIFY_STRUCT_FIELD_HELP_TEXT: &str =
    "Struct field's type must match the type specified in its declaration.";

pub(crate) fn struct_instantiation(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    mut call_path_binding: TypeBinding<CallPath>,
    fields: &[StructExpressionField],
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
        inner: CallPath { suffix, .. },
        type_arguments,
        span: inner_span,
    } = &call_path_binding;

    if let TypeArgs::Prefix(_) = type_arguments {
        return Err(
            handler.emit_err(CompileError::DoesNotTakeTypeArgumentsAsPrefix {
                name: suffix.clone(),
                span: type_arguments.span(),
            }),
        );
    }

    let type_arguments = type_arguments.to_vec();

    // We first create a custom type and then resolve it to the struct type.
    let custom_type_id = match (suffix.as_str(), type_arguments.is_empty()) {
        ("Self", true) => type_engine.new_self_type(engines, suffix.span()),
        ("Self", false) => {
            return Err(handler.emit_err(CompileError::TypeArgumentsNotAllowed {
                span: suffix.span(),
            }));
        }
        (_, true) => type_engine.new_custom_from_name(engines, suffix.clone()),
        (_, false) => type_engine.new_custom(engines, suffix.clone().into(), Some(type_arguments)),
    };

    // find the module that the struct decl is in
    let type_info_prefix = call_path_binding
        .inner
        .to_fullpath(engines, ctx.namespace())
        .prefixes;
    ctx.namespace()
        .require_module_from_absolute_path(handler, &type_info_prefix)?;

    // resolve the type of the struct decl
    let type_id = ctx
        .resolve_type(
            handler,
            custom_type_id,
            inner_span,
            EnforceTypeArguments::No,
            Some(&type_info_prefix),
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

    // extract the struct name and fields from the type info
    let type_info = type_engine.get(type_id);
    let struct_id = type_info.expect_struct(handler, engines, &span)?;
    let struct_decl = decl_engine.get_struct(&struct_id);

    let (struct_can_be_changed, is_public_struct_access) =
        StructAccessInfo::get_info(engines, &struct_decl, ctx.namespace()).into();
    let struct_has_private_fields = struct_decl.has_private_fields();
    let struct_can_be_instantiated = !is_public_struct_access || !struct_has_private_fields;
    let all_fields_are_private = struct_decl.has_only_private_fields();
    let struct_is_empty = struct_decl.is_empty();
    let struct_name = struct_decl.call_path.suffix.clone();
    let struct_decl_span = struct_decl.span();

    // Before we do the type check, let's first check for the field related errors (privacy issues, non-existing fields, ...).
    // These errors are independent of the type check, so we can collect all of them and then proceed with the type check.

    // To avoid conflicting and overlapping errors, we follow the Rust approach:
    // - Missing fields are reported only if the struct can actually be instantiated.
    // - Individual fields issues are always reported: private field access, non-existing fields.
    let struct_fields = &struct_decl.fields;

    if !struct_can_be_instantiated {
        let constructors = collect_struct_constructors(
            ctx.namespace(),
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

    // Check that there are no duplicate fields.
    let mut seen_fields: BTreeSet<Ident> = BTreeSet::new();
    for field in fields.iter() {
        if let Some(duplicate) = seen_fields.get(&field.name) {
            handler.emit_err(CompileError::StructFieldDuplicated {
                field_name: field.name.clone(),
                duplicate: duplicate.clone(),
            });
        }
        seen_fields.insert(field.name.clone());
    }

    // Check that there are no extra fields.
    for field in fields.iter() {
        if !struct_fields.iter().any(|x| x.name == field.name) {
            handler.emit_err(CompileError::StructFieldDoesNotExist {
                field_name: (&field.name).into(), // Explicit borrow to force the `From<&BaseIdent>` instead of `From<BaseIdent>`.
                available_fields: TyStructField::accessible_fields_names(
                    struct_fields,
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
        for field in fields.iter() {
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

    // Type check the fields and the struct.

    // If the context type annotation is a struct that can coerce into the struct to instantiate,
    // use the type coming from the context type annotation for type checking.
    // We do this to likely get a more specific type from the type annotation, although this must
    // not be the case. At the end, we will "merge" the struct type coming from the context and
    // from the struct to instantiate to cover cases like, e.g., this one:
    //
    //   let _: Struct<u8, _, _> = Struct<_, bool, u32> { x: 123, y: true, z: 456 };
    //
    // Not that, until we separate type checking and type inference phase, and do the inference
    // based on the overall scope, this is the best we can do to cover the largest variety of cases.
    //
    // If the context type annotation is not a struct that can coerce into the struct to instantiate,
    // take the struct type coming from the struct instantiation as the expected type.
    // This means that a type-mismatch error will be generated up the type-checking chain between
    // the instantiated struct type and the expected type, but the struct instantiation itself must
    // not necessarily be erroneous. (Examples are given below.)
    //
    // We also want to adjust the help message accordingly, depending where the type expectation is
    // coming from.
    //
    // E.g.:
    //   let _: Struct<u8> = Struct { x: 123 }; // Ok.
    //   let _: Struct<u8> = Struct { x: 123u64 };
    //                                   ^^^^^^ Expected `u8` found `u64`.
    //                                   ^^^^^^ Must match **variable** declaration.
    //   let _: Struct<u8> = Struct<bool> { x: true };
    //                       ^^^^^^^^^^^^^^^^^^^^^^^^ Expected `Struct<u8>` found `Struct<bool>`. (But `true` is ok.)
    //                       ^^^^^^^^^^^^^^^^^^^^^^^^ Must match **variable** declaration.
    //   let _: Struct<u8> = Struct<bool> { x: "not bool" };
    //                                         ^^^^^^^^^^ Expected `bool` found `str`.
    //                                         ^^^^^^^^^^ Must match **struct** declaration.
    let context_expected_type_id = type_engine.get_unaliased_type_id(ctx.type_annotation());
    let (is_context_type_used, type_check_struct_decl, help_text) =
        match &*type_engine.get(context_expected_type_id) {
            TypeInfo::Struct(decl_id) => {
                let context_expected_struct_decl = decl_engine.get_struct(decl_id);
                if UnifyCheck::coercion(engines)
                    .check_structs(&context_expected_struct_decl, &struct_decl)
                {
                    (true, context_expected_struct_decl, ctx.help_text())
                } else {
                    (false, struct_decl.clone(), UNIFY_STRUCT_FIELD_HELP_TEXT)
                }
            }
            _ => (false, struct_decl.clone(), UNIFY_STRUCT_FIELD_HELP_TEXT),
        };

    let typed_fields = type_check_field_arguments(
        handler,
        ctx.by_ref(),
        &struct_name,
        fields,
        &type_check_struct_decl.fields,
        &span,
        &struct_decl_span,
        help_text,
        // Emit the missing fields error only if the struct can actually be instantiated.
        struct_can_be_instantiated,
    )?;

    // The above type check will unify the types behind the `type_check_struct_decl.fields`
    // and the resulting expression types coming from `fields`.
    // But if the struct coming from the context was used for the unification, we
    // still need to unify the resulting struct type.
    if is_context_type_used {
        // Let's unify just the struct fields first, to be able to locate the error
        // message to each individual initialization value, because that's where the issue is.
        unify_field_arguments_and_struct_fields(
            handler,
            ctx.engines(),
            &typed_fields,
            &struct_decl.fields,
            help_text,
        )?;

        // Then let's unify the struct types.
        // Note that, in this case, the type we are actually expecting is the `type_id` and the
        // type which was provided by the context is the one we see as received, because we did
        // the previous type unification based on that type.
        // Short-circuit if the unification fails, by checking if the scoped handler
        // has collected any errors.
        handler.scope(|handler| {
            type_engine.unify_with_generic(
                handler,
                engines,
                context_expected_type_id,
                type_id,
                &span,
                help_text,
                None,
            );
            Ok(())
        })?;
    }

    let instantiation_span = inner_span.clone();
    ctx.with_generic_shadowing_mode(GenericShadowingMode::Allow)
        .scoped(handler, None, |scoped_ctx| {
            // Insert struct type parameter into namespace.
            // This is required so check_type_parameter_bounds can resolve generic trait type parameters.
            for p in struct_decl
                .generic_parameters
                .iter()
                .filter_map(|x| x.as_type_parameter())
            {
                p.insert_into_namespace_self(handler, scoped_ctx.by_ref())?;
            }

            type_id.check_type_parameter_bounds(handler, scoped_ctx.by_ref(), &span, None)?;

            let exp = ty::TyExpression {
                expression: ty::TyExpressionVariant::StructExpression {
                    struct_id,
                    fields: typed_fields,
                    instantiation_span,
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
    namespace.current_module().read(engines, |m| {
        m.get_items_for_type(engines, struct_type_id)
            .iter()
            .filter_map(|item| match item {
                ResolvedTraitImplItem::Parsed(_) => unreachable!(),
                ResolvedTraitImplItem::Typed(item) => match item {
                    ty::TyTraitItem::Fn(fn_decl_id) => {
                        Some(fn_decl_id.get_method_safe_to_unify(engines, struct_type_id))
                    }
                    _ => None,
                },
            })
            .map(|fn_decl_id| engines.de().get_function(&fn_decl_id))
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
    })
}

/// Type checks the field arguments.
#[allow(clippy::too_many_arguments)]
fn type_check_field_arguments(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    struct_name: &Ident,
    fields: &[StructExpressionField],
    struct_fields: &[ty::TyStructField],
    span: &Span,
    struct_decl_span: &Span,
    help_text: &'static str,
    emit_missing_fields_error: bool,
) -> Result<Vec<ty::TyStructExpressionField>, ErrorEmitted> {
    handler.scope(|handler| {
        let type_engine = ctx.engines.te();

        let mut typed_fields = vec![];
        let mut missing_fields = vec![];

        for struct_field in struct_fields.iter() {
            match fields.iter().find(|x| x.name == struct_field.name) {
                Some(field) => {
                    let ctx = ctx
                        .by_ref()
                        .with_help_text(help_text)
                        .with_type_annotation(struct_field.type_argument.type_id())
                        .with_unify_generic(true);

                    // TODO: Remove the `handler.scope` once https://github.com/FuelLabs/sway/issues/5606 gets solved.
                    //       We need it here so that we can short-circuit in case of a `TypeMismatch` error which is
                    //       not treated as an error in the `type_check()`'s result.
                    let typed_expr = handler
                        .scope(|handler| ty::TyExpression::type_check(handler, ctx, &field.value));

                    let value = match typed_expr {
                        Ok(res) => res,
                        Err(_) => continue,
                    };

                    typed_fields.push(ty::TyStructExpressionField {
                        value,
                        name: field.name.clone(),
                    });
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
                            return_type: type_engine.id_of_error_recovery(err),
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
    })
}

/// Unifies the field arguments and the types of the fields from the struct
/// definition.
fn unify_field_arguments_and_struct_fields(
    handler: &Handler,
    engines: &Engines,
    typed_fields: &[ty::TyStructExpressionField],
    struct_fields: &[ty::TyStructField],
    help_text: &str,
) -> Result<(), ErrorEmitted> {
    let type_engine = engines.te();

    handler.scope(|handler| {
        for struct_field in struct_fields.iter() {
            if let Some(typed_field) = typed_fields.iter().find(|x| x.name == struct_field.name) {
                type_engine.unify_with_generic(
                    handler,
                    engines,
                    typed_field.value.return_type,
                    struct_field.type_argument.type_id(),
                    &typed_field.value.span, // Use the span of the initialization value.
                    help_text,
                    None,
                );
            }
        }
        Ok(())
    })
}
