use generational_arena::Index;

use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::*;
use crate::semantic_analysis::{ast_node::*, TCOpts, TypeCheckArguments};
use crate::type_engine::{look_up_type_id, TypeId};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_enum(
    module: Index,
    enum_decl: TypedEnumDeclaration,
    call_path: CallPath,
    args: Vec<Expression>,
    type_arguments: Vec<TypeArgument>,
    namespace: NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let instantiation_span = call_path.span();
    let enum_field_name = call_path.suffix;

    // monomorphize the enum definition with the type arguments
    let enum_decl = check!(
        enum_decl.monomorphize(
            type_arguments,
            false,
            &module,
            Some(self_type),
            Some(&instantiation_span)
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

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
                return_type: enum_decl.type_id(),
                expression: TypedExpressionVariant::EnumInstantiation {
                    tag,
                    contents: None,
                    enum_decl,
                    variant_name,
                    instantiation_span: instantiation_span.clone(),
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
                    return_type: enum_decl.type_id(),
                    expression: TypedExpressionVariant::EnumInstantiation {
                        tag,
                        contents: Some(Box::new(typed_expr)),
                        enum_decl,
                        variant_name,
                        instantiation_span: instantiation_span.clone(),
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
