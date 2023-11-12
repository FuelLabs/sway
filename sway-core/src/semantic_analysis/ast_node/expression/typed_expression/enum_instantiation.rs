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

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [ty::TyExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    enum_ref: DeclRefEnum,
    enum_variant_name: Ident,
    args_opt: Option<Vec<Expression>>,
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
    // args_opt.is_some() returns true when this variant was called with parenthesis.
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

    match (
        &args[..],
        type_engine.get(enum_variant.type_argument.type_id),
    ) {
        ([], ty) if ty.is_unit() => Ok(ty::TyExpression {
            return_type: type_engine.insert(
                engines,
                TypeInfo::Enum(enum_ref.clone()),
                enum_ref.span().source_id(),
            ),
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
            let enum_ctx = ctx
                .by_ref()
                .with_help_text("Enum instantiator must match its declared variant type.")
                .with_type_annotation(enum_variant.type_argument.type_id);
            let typed_expr = ty::TyExpression::type_check(handler, enum_ctx, single_expr.clone())?;

            // unify the value of the argument with the variant
            handler.scope(|handler| {
                type_engine.unify(
                    handler,
                    engines,
                    typed_expr.return_type,
                    enum_variant.type_argument.type_id,
                    &typed_expr.span,
                    "Enum instantiator must match its declared variant type.",
                    None,
                );
                Ok(())
            })?;

            // we now know that the instantiator type matches the declared type, via the above tpe
            // check

            let type_id = type_engine.insert(
                engines,
                TypeInfo::Enum(enum_ref.clone()),
                enum_ref.span().source_id(),
            );

            type_id.check_type_parameter_bounds(handler, ctx, &enum_variant_name.span(), vec![])?;

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
