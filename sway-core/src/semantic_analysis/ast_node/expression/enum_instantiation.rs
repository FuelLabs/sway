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
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];
    // if this is a generic enum, i.e. it has some type
    // parameters, monomorphize it before unifying the
    // types
    let enum_decl = if enum_decl.type_parameters.is_empty() {
        enum_decl
    } else {
        enum_decl.monomorphize()
    };
    let (enum_field_type, tag, variant_name) = match enum_decl
        .variants
        .iter()
        .find(|x| x.name.as_str() == enum_field_name.as_str())
    {
        Some(o) => (o.r#type, o.tag, o.name.clone()),
        None => {
            errors.push(CompileError::UnknownEnumVariant {
                enum_name: enum_decl.name.clone(),
                variant_name: enum_field_name.clone(),
                span: enum_field_name.span().clone(),
            });
            return err(warnings, errors);
        }
    };

    // If there is an instantiator, it must match up with the type. If there is not an
    // instantiator, then the type of the enum is necessarily the unit type.

    match (&args[..], look_up_type_id(enum_field_type)) {
        ([], ty) if ty.is_unit() => ok(
            TypedExpression {
                return_type: enum_decl.as_type(),
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag,
                    contents: None,
                    enum_decl,
                    variant_name,
                },
                is_constant: IsConstant::No,
                span: enum_field_name.span().clone(),
            },
            warnings,
            errors,
        ),
        ([single_expr], _type) => {
            let typed_expr = check!(
                TypedExpression::type_check(TypeCheckArguments {
                    checkee: single_expr.clone(),
                    namespace,
                    crate_namespace,
                    return_type_annotation: enum_field_type,
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
                    return_type: enum_decl.as_type(),
                    expression: TypedExpressionVariant::EnumInstantiation {
                        tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name,
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
