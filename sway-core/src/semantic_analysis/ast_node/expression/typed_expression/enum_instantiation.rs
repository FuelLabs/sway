use crate::{
    decl_engine::DeclRefEnum,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [ty::TyExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    mut ctx: TypeCheckContext,
    enum_ref: DeclRefEnum,
    enum_variant_name: Ident,
    args_opt: Option<Vec<Expression>>,
    call_path_binding: TypeBinding<CallPath>,
    call_path_decl: ty::TyDecl,
    span: &Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    let enum_decl = decl_engine.get_enum(&enum_ref);
    let enum_variant = check!(
        enum_decl
            .expect_variant_from_name(&enum_variant_name)
            .cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Return an error if enum variant is of type unit and it is called with parenthesis.
    // args_opt.is_some() returns true when this variant was called with parenthesis.
    if type_engine
        .get(enum_variant.type_argument.initial_type_id)
        .is_unit()
        && args_opt.is_some()
    {
        errors.push(CompileError::UnitVariantWithParenthesesEnumInstantiator {
            span: enum_variant_name.span(),
            ty: enum_variant.name.as_str().to_string(),
        });
        return err(warnings, errors);
    }
    let args = args_opt.unwrap_or_default();

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (
        &args[..],
        type_engine.get(enum_variant.type_argument.type_id),
    ) {
        ([], ty) if ty.is_unit() => ok(
            ty::TyExpression {
                return_type: type_engine.insert(decl_engine, TypeInfo::Enum(enum_ref.clone())),
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
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let enum_ctx = ctx
                .by_ref()
                .with_help_text("Enum instantiator must match its declared variant type.")
                .with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));
            let typed_expr = check!(
                ty::TyExpression::type_check(enum_ctx, single_expr.clone()),
                return err(warnings, errors),
                warnings,
                errors
            );

            // unify the value of the argument with the variant
            check!(
                CompileResult::from(type_engine.unify(
                    decl_engine,
                    typed_expr.return_type,
                    enum_variant.type_argument.type_id,
                    span,
                    "Enum instantiator must match its declared variant type.",
                    None
                )),
                return err(warnings, errors),
                warnings,
                errors
            );

            // we now know that the instantiator type matches the declared type, via the above tpe
            // check

            let type_id = type_engine.insert(decl_engine, TypeInfo::Enum(enum_ref.clone()));

            check!(
                type_id.check_type_parameter_bounds(&ctx, &enum_variant_name.span()),
                return err(warnings, errors),
                warnings,
                errors
            );

            ok(
                ty::TyExpression {
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
                },
                warnings,
                errors,
            )
        }
        ([], _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: enum_variant_name.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) if ty.is_unit() => {
            errors.push(CompileError::UnnecessaryEnumInstantiator {
                span: enum_variant_name.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) => {
            errors.push(CompileError::MoreThanOneEnumInstantiator {
                span: enum_variant_name.span(),
                ty: engines.help_out(ty).to_string(),
            });
            err(warnings, errors)
        }
    }
}
