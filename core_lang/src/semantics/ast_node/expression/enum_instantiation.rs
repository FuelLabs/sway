use crate::error::*;
use crate::semantics::ast_node::*;

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
pub(crate) fn instantiate_enum<'sc>(
    enum_decl: TypedEnumDeclaration<'sc>,
    enum_field_name: Ident<'sc>,
    instantiator: Option<Box<Expression<'sc>>>,
    type_arguments: Vec<ResolvedType<'sc>>,
    namespace: &Namespace<'sc>,
) -> CompileResult<'sc, TypedExpression<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let enum_decl = type_check!(
        enum_decl.resolve_generic_types(type_arguments),
        return err(warnings, errors),
        warnings,
        errors
    );
    let (enum_field_type, tag, variant_name) = match enum_decl
        .variants
        .iter()
        .find(|x| x.name.primary_name == enum_field_name.primary_name)
    {
        Some(o) => (o.r#type.clone(), o.tag, o.name.clone()),
        None => {
            errors.push(CompileError::UnknownEnumVariant {
                enum_name: enum_decl.name.primary_name,
                variant_name: enum_field_name.primary_name,
                span: enum_field_name.clone().span,
            });
            return err(warnings, errors);
        }
    };

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (instantiator, enum_field_type) {
        (None, ResolvedType::Unit) => ok(
            TypedExpression {
                return_type: ResolvedType::Unit,
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag,
                    contents: None,
                    enum_decl,
                    variant_name,
                },
                is_constant: IsConstant::No,
                span: enum_field_name.span.clone(),
            },
            warnings,
            errors,
        ),
        (Some(boxed_expr), r#type) => {
            let typed_expr = type_check!(
                TypedExpression::type_check(
                    *boxed_expr,
                    namespace,
                    Some(r#type.clone()),
                    "Enum instantiator must match its declared variant type."
                ),
                return err(warnings, errors),
                warnings,
                errors
            );

            // we now know that the instantiator type matches the declared type, via the above tpe
            // check

            ok(
                TypedExpression {
                    return_type: enum_decl.as_type(),
                    expression: TypedExpressionVariant::EnumInstantiation {
                        tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name,
                    },
                    is_constant: IsConstant::No,
                    span: enum_field_name.span.clone(),
                },
                warnings,
                errors,
            )
        }
        (None, _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: enum_field_name.span.clone(),
            });
            return err(warnings, errors);
        }
    }
}
