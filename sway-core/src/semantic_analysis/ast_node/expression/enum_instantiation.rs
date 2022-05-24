use ast_node::declaration::CreateTypeId;

use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::*;
use crate::semantic_analysis::{ast_node::*, TCOpts, TypeCheckArguments};
use crate::type_engine::{look_up_type_id, TypeId};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    enum_decl: TypedEnumDeclaration,
    enum_field_name: Ident,
    args: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    namespace: &mut Namespace,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // monomorphize the enum definition with the type arguments
    let enum_decl = check!(
        namespace.monomorphize(
            enum_decl,
            type_arguments,
            EnforceTypeArguments::No,
            Some(self_type),
            Some(enum_field_name.span())
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

    match (&args[..], look_up_type_id(enum_variant.r#type)) {
        ([], ty) if ty.is_unit() => ok(
            TypedExpression {
                return_type: enum_decl.create_type_id(),
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag: enum_variant.tag,
                    contents: None,
                    enum_decl,
                    variant_name: enum_variant.name,
                    instantiation_span: enum_field_name.span().clone(),
                },
                is_constant: IsConstant::No,
                span: enum_field_name.span().clone(),
            },
            warnings,
            errors,
        ),
        ([single_expr], _) => {
            let typed_expr = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: single_expr.clone(),
                    namespace,
                    return_type_annotation: enum_variant.r#type,
                    help_text: "Enum instantiator must match its declared variant type.",
                    self_type,
                    build_config,
                    dead_code_graph,
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
                        instantiation_span: enum_field_name.span().clone(),
                    },
                    is_constant: IsConstant::No,
                    span: enum_field_name.span().clone(),
                },
                warnings,
                errors,
            )
        }
        ([], _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: enum_field_name.span().clone(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) if ty.is_unit() => {
            errors.push(CompileError::UnnecessaryEnumInstantiator {
                span: enum_field_name.span().clone(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) => {
            errors.push(CompileError::MoreThanOneEnumInstantiator {
                span: enum_field_name.span().clone(),
                ty: ty.friendly_type_str(),
            });
            err(warnings, errors)
        }
    }
}
