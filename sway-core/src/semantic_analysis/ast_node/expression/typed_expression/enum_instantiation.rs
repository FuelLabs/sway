use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
};

use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [ty::TyExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    ctx: TypeCheckContext,
    enum_decl: ty::TyEnumDeclaration,
    enum_name: Ident,
    enum_variant_name: Ident,
    args: Vec<Expression>,
    span: &Span,
) -> CompileResult<ty::TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let type_engine = ctx.type_engine;
    let decl_engine = ctx.decl_engine;
    let engines = ctx.engines();

    let enum_variant = check!(
        enum_decl
            .expect_variant_from_name(&enum_variant_name)
            .cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (&args[..], type_engine.look_up_type_id(enum_variant.type_id)) {
        ([], ty) if ty.is_unit() => ok(
            ty::TyExpression {
                return_type: enum_decl.create_type_id(engines),
                expression: ty::TyExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: None,
                    enum_decl,
                    variant_name: enum_variant.name,
                    enum_instantiation_span: enum_name.span(),
                    variant_instantiation_span: enum_variant_name.span(),
                },
                span: enum_variant_name.span(),
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let ctx = ctx
                .with_help_text("Enum instantiator must match its declared variant type.")
                .with_type_annotation(type_engine.insert_type(decl_engine, TypeInfo::Unknown));
            let typed_expr = check!(
                ty::TyExpression::type_check(ctx, single_expr.clone()),
                return err(warnings, errors),
                warnings,
                errors
            );

            // unify the value of the argument with the variant
            check!(
                CompileResult::from(type_engine.unify_adt(
                    decl_engine,
                    typed_expr.return_type,
                    enum_variant.type_id,
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

            ok(
                ty::TyExpression {
                    return_type: enum_decl.create_type_id(engines),
                    expression: ty::TyExpressionVariant::EnumInstantiation {
                        tag: enum_variant.tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name: enum_variant.name,
                        enum_instantiation_span: enum_name.span(),
                        variant_instantiation_span: enum_variant_name.span(),
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
