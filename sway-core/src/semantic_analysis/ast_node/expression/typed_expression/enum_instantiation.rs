use crate::{error::*, language::parse_tree::*, semantic_analysis::*, type_system::*};

use sway_types::{Ident, Spanned};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TyExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    ctx: TypeCheckContext,
    enum_decl: TyEnumDeclaration,
    enum_name: Ident,
    enum_variant_name: Ident,
    args: Vec<Expression>,
) -> CompileResult<TyExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

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

    match (&args[..], look_up_type_id(enum_variant.type_id)) {
        ([], ty) if ty.is_unit() => ok(
            TyExpression {
                return_type: enum_decl.create_type_id(),
                expression: TyExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: None,
                    enum_decl,
                    variant_name: enum_variant.name,
                    enum_instantiation_span: enum_name.span(),
                    variant_instantiation_span: enum_variant_name.span(),
                },
                is_constant: IsConstant::No,
                span: enum_variant_name.span(),
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let ctx = ctx
                .with_help_text("Enum instantiator must match its declared variant type.")
                .with_type_annotation(enum_variant.type_id);
            let typed_expr = check!(
                TyExpression::type_check(ctx, single_expr.clone()),
                return err(warnings, errors),
                warnings,
                errors
            );

            // we now know that the instantiator type matches the declared type, via the above tpe
            // check

            ok(
                TyExpression {
                    return_type: enum_decl.create_type_id(),
                    expression: TyExpressionVariant::EnumInstantiation {
                        tag: enum_variant.tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name: enum_variant.name,
                        enum_instantiation_span: enum_name.span(),
                        variant_instantiation_span: enum_variant_name.span(),
                    },
                    is_constant: IsConstant::No,
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
                ty: ty.to_string(),
            });
            err(warnings, errors)
        }
    }
}
