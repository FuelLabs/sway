use crate::{error::*, parse_tree::*, semantic_analysis::*, type_engine::*};

use sway_types::{Ident, Spanned};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    mut ctx: TypeCheckContext,
    enum_decl: TypedEnumDeclaration,
    enum_field_name: Ident,
    args: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // monomorphize the enum definition with the type arguments
    let enum_decl = check!(
        ctx.monomorphize(
            enum_decl,
            type_arguments,
            EnforceTypeArguments::No,
            Some(&enum_field_name.span())
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    let enum_variant = check!(
        enum_decl
            .expect_variant_from_name(&enum_field_name)
            .cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (&args[..], look_up_type_id(enum_variant.type_id)) {
        ([], ty) if ty.is_unit() => ok(
            TypedExpression {
                return_type: enum_decl.create_type_id(),
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: None,
                    enum_decl,
                    variant_name: enum_variant.name,
                    instantiation_span: enum_field_name.span(),
                },
                is_constant: IsConstant::No,
                span: enum_field_name.span(),
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let ctx = ctx
                .with_help_text("Enum instantiator must match its declared variant type.")
                .with_type_annotation(enum_variant.type_id);
            let typed_expr = check!(
                TypedExpression::type_check(ctx, single_expr.clone()),
                return err(warnings, errors),
                warnings,
                errors
            );

            // we now know that the instantiator type matches the declared type, via the above tpe
            // check

            ok(
                TypedExpression {
                    return_type: enum_decl.create_type_id(),
                    expression: TypedExpressionVariant::EnumInstantiation {
                        tag: enum_variant.tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name: enum_variant.name,
                        instantiation_span: enum_field_name.span(),
                    },
                    is_constant: IsConstant::No,
                    span: enum_field_name.span(),
                },
                warnings,
                errors,
            )
        }
        ([], _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: enum_field_name.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) if ty.is_unit() => {
            errors.push(CompileError::UnnecessaryEnumInstantiator {
                span: enum_field_name.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) => {
            errors.push(CompileError::MoreThanOneEnumInstantiator {
                span: enum_field_name.span(),
                ty: ty.to_string(),
            });
            err(warnings, errors)
        }
    }
}
