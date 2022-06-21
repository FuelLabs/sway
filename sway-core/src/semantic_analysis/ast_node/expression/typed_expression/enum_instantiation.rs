use crate::{error::*, parse_tree::*, semantic_analysis::*, type_engine::*};

use sway_types::{Span, Spanned};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    enum_decl: TypedEnumDeclaration,
    type_binding: TypeBinding<CallPath>,
    arguments: Vec<Expression>,
    namespace: &mut Namespace,
    self_type: TypeId,
    opts: TCOpts,
    span: Span,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // monomorphize the enum definition with the type arguments
    let enum_decl = check!(
        namespace.monomorphize(
            enum_decl,
            type_binding.type_arguments.clone(),
            EnforceTypeArguments::No,
            Some(self_type),
            Some(&type_binding.span())
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    let enum_variant = check!(
        enum_decl
            .expect_variant_from_name(&type_binding.inner.suffix)
            .cloned(),
        return err(warnings, errors),
        warnings,
        errors
    );

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (&arguments[..], look_up_type_id(enum_variant.type_id)) {
        ([], ty) if ty.is_unit() => ok(
            TypedExpression {
                return_type: enum_decl.create_type_id(),
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: None,
                    enum_decl,
                    variant_name: enum_variant.name,
                    instantiation_span: type_binding.span(),
                },
                is_constant: IsConstant::No,
                span,
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let typed_expr = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: single_expr.clone(),
                    namespace,
                    return_type_annotation: enum_variant.type_id,
                    help_text: "Enum instantiator must match its declared variant type.",
                    self_type,
                    mode: Mode::NonAbi,
                    opts,
                }),
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
                        instantiation_span: type_binding.span(),
                    },
                    is_constant: IsConstant::No,
                    span,
                },
                warnings,
                errors,
            )
        }
        ([], _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: type_binding.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) if ty.is_unit() => {
            errors.push(CompileError::UnnecessaryEnumInstantiator {
                span: type_binding.span(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) => {
            errors.push(CompileError::MoreThanOneEnumInstantiator {
                span: type_binding.span(),
                ty: ty.to_string(),
            });
            err(warnings, errors)
        }
    }
}
