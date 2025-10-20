use crate::{
    decl_engine::DeclRefEnum,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Spanned};

const UNIFY_ENUM_VARIANT_HELP_TEXT: &str =
    "Enum instantiator must match its declared variant type.";

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [ty::TyExpression] of variant [ty::TyExpressionVariant::EnumInstantiation].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    enum_ref: DeclRefEnum,
    enum_variant_name: Ident,
    args_opt: Option<&[Expression]>,
    call_path_binding: TypeBinding<CallPath>,
    call_path_decl: ty::TyDecl,
) -> Result<ty::TyExpression, ErrorEmitted> {
    let type_engine = ctx.engines.te();
    let decl_engine = ctx.engines.de();
    let engines = ctx.engines();

    let enum_decl = decl_engine.get_enum(&enum_ref);
    let enum_variant = enum_decl
        .expect_variant_from_name(handler, &enum_variant_name)
        .cloned()?;

    // Return an error if enum variant is of type unit and it is called with parenthesis.
    // `args_opt.is_some()` returns true when this variant was called with parenthesis.
    if type_engine
        .get(enum_variant.type_argument.initial_type_id)
        .is_unit()
        && args_opt.is_some()
    {
        return Err(
            handler.emit_err(CompileError::UnitVariantWithParenthesesEnumInstantiator {
                span: enum_variant_name.span(),
                ty: enum_variant.name.as_str().to_string(),
            }),
        );
    }
    let args = args_opt.unwrap_or_default();

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (&args, &*type_engine.get(enum_variant.type_argument.type_id)) {
        ([], ty) if ty.is_unit() => Ok(ty::TyExpression {
            return_type: type_engine.insert_enum(engines, *enum_ref.id()),
            expression: ty::TyExpressionVariant::EnumInstantiation {
                tag: enum_variant.tag,
                contents: None,
                enum_ref,
                variant_name: enum_variant.name,
                variant_instantiation_span: enum_variant_name.span(),
                call_path_binding,
                call_path_decl,
            },
            span: enum_variant_name.span(),
        }),
        ([single_expr], _) => {
            // If the context type annotation is an enum that can coerce into the enum to instantiate,
            // force `single_expr` to be of the enum variant type coming from the context type annotation,
            // We do this to likely get a more specific type from the type annotation, although this must
            // not be the case. At the end, we will "merge" the enum type coming from the context and
            // from the enum to instantiate to cover cases like, e.g., this one:
            //
            //   let _: Enum<u8, _, _> = Enum::<_, bool, u32>::A(123);
            //
            // Not that, until we separate type checking and type inference phase, and do the inference
            // based on the overall scope, this is the best we can do to cover the largest variety of cases.
            //
            // If the context type annotation is not an enum that can coerce into the enum to instantiate,
            // take the enum variant type coming from the enum declaration as the expected type.
            // This means that a type-mismatch error will be generated up the type-checking chain between
            // the instantiated enum type and the expected type, but the enum instantiation itself must
            // not necessarily be erroneous. (Examples are given below.)
            //
            // We also want to adjust the help message accordingly, depending where the type expectation is
            // coming from.
            //
            // E.g.:
            //   let _: Option<u8> = Option::Some(123); // Ok.
            //   let _: Option<u8> = Option::Some(123u64);
            //                                    ^^^^^^ Expected `u8` found `u64`.
            //                                    ^^^^^^ Must match **variable** declaration.
            //   let _: Option<u8> = Option::Some::<bool>(true);
            //                       ^^^^^^^^^^^^^^^^^^^^^^^^^^ Expected `Option<u8>` found `Option<bool>`. (But `true` is ok.)
            //                       ^^^^^^^^^^^^^^^^^^^^^^^^^^ Must match **variable** declaration.
            //   let _: Option<u8> = Option::Some::<bool>("not bool");
            //                                            ^^^^^^^^^^ Expected `bool` found `str`.
            //                                            ^^^^^^^^^^ Must match **enum** declaration.
            let context_expected_type_id = type_engine.get_unaliased_type_id(ctx.type_annotation());
            let (is_context_type_used, enum_variant_type_id, help_text) =
                match &*type_engine.get(context_expected_type_id) {
                    TypeInfo::Enum(e) => {
                        let context_expected_enum_decl = decl_engine.get_enum(e);
                        if UnifyCheck::coercion(engines)
                            .check_enums(&context_expected_enum_decl, &enum_decl)
                        {
                            let context_expected_enum_variant = context_expected_enum_decl
                                .expect_variant_from_name(handler, &enum_variant_name)
                                .cloned()?;
                            (
                                true,
                                context_expected_enum_variant.type_argument.type_id,
                                ctx.help_text(),
                            )
                        } else {
                            (
                                false,
                                enum_variant.type_argument.type_id,
                                UNIFY_ENUM_VARIANT_HELP_TEXT,
                            )
                        }
                    }
                    _ => (
                        false,
                        enum_variant.type_argument.type_id,
                        UNIFY_ENUM_VARIANT_HELP_TEXT,
                    ),
                };

            let enum_ctx = ctx
                .by_ref()
                .with_help_text(help_text)
                .with_type_annotation(enum_variant_type_id);

            // TODO: Remove the `handler.scope` once https://github.com/FuelLabs/sway/issues/5606 gets solved.
            //       We need it here so that we can short-circuit in case of a `TypeMismatch` error which is
            //       not treated as an error in the `type_check()`'s result.
            let typed_expr = handler
                .scope(|handler| ty::TyExpression::type_check(handler, enum_ctx, single_expr))?;

            // Create the resulting enum type based on the enum we have instantiated.
            // Note that we clone the `enum_ref` but the unification we do below will
            // affect the types behind that new enum decl reference.
            let type_id = type_engine.insert_enum(engines, *enum_ref.id());

            // The above type check will unify the type behind the `enum_variant_type_id`
            // and the resulting expression type.
            // But if the enum coming from the context was used for the unification, we
            // still need to unify the resulting enum type.
            if is_context_type_used {
                // Let's unify just the variant type first, to be able to locate the error
                // message to the instantiator, because that's where the issue is.
                // Short-circuit if the unification fails, by checking if the scoped handler
                // has collected any errors.
                handler.scope(|handler| {
                    type_engine.unify(
                        handler,
                        engines,
                        typed_expr.return_type,
                        enum_variant.type_argument.type_id,
                        &single_expr.span, // Use the span of the instantiator expression.
                        help_text,
                        || None,
                    );
                    Ok(())
                })?;

                // Then let's unify the enum types.
                // Note that, in this case, the type we are actually expecting is the `type_id` and the
                // type which was provided by the context is the one we see as received, because we did
                // the previous type unification based on that type.
                handler.scope(|handler| {
                    type_engine.unify(
                        handler,
                        engines,
                        context_expected_type_id,
                        type_id,
                        &enum_variant_name.span(),
                        help_text,
                        || None,
                    );
                    Ok(())
                })?;
            }

            type_id.check_type_parameter_bounds(handler, ctx, &enum_variant_name.span(), None)?;

            Ok(ty::TyExpression {
                return_type: type_id,
                expression: ty::TyExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: Some(Box::new(typed_expr)),
                    enum_ref,
                    variant_name: enum_variant.name,
                    variant_instantiation_span: enum_variant_name.span(),
                    call_path_binding,
                    call_path_decl,
                },
                span: enum_variant_name.span(),
            })
        }
        ([], _) => Err(handler.emit_err(CompileError::MissingEnumInstantiator {
            span: enum_variant_name.span(),
        })),
        (_too_many_expressions, ty) if ty.is_unit() => {
            Err(handler.emit_err(CompileError::UnnecessaryEnumInstantiator {
                span: enum_variant_name.span(),
            }))
        }
        (_too_many_expressions, ty) => {
            Err(handler.emit_err(CompileError::MoreThanOneEnumInstantiator {
                span: enum_variant_name.span(),
                ty: engines.help_out(ty).to_string(),
            }))
        }
    }
}
