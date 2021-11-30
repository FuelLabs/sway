use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::*;
use crate::semantic_analysis::ast_node::*;
use crate::type_engine::{look_up_type_id, TypeId};

/// Given an enum declaration and the instantiation expression/type arguments, construct a valid
/// [TypedExpression].
pub(crate) fn instantiate_enum<'sc>(
    enum_decl: TypedEnumDeclaration<'sc>,
    enum_field_name: Ident<'sc>,
    args: Vec<Expression<'sc>>,
    namespace: &mut Namespace<'sc>,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompileResult<'sc, TypedExpression<'sc>> {
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
        .find(|x| x.name.primary_name == enum_field_name.primary_name)
    {
        Some(o) => (o.r#type, o.tag, o.name.clone()),
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

    match (&args[..], look_up_type_id(enum_field_type)) {
        ([], TypeInfo::Unit) => ok(
            TypedExpression {
                return_type: enum_decl.as_type(),
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
        ([single_expr], _type) => {
            let typed_expr = check!(
                TypedExpression::type_check(
                    single_expr.clone(),
                    namespace,
                    Some(enum_field_type),
                    "Enum instantiator must match its declared variant type.",
                    self_type,
                    build_config,
                    dead_code_graph,
                    dependency_graph,
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
        ([], _) => {
            errors.push(CompileError::MissingEnumInstantiator {
                span: enum_field_name.span.clone(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, TypeInfo::Unit) => {
            errors.push(CompileError::UnnecessaryEnumInstantiator {
                span: enum_field_name.span.clone(),
            });
            err(warnings, errors)
        }
        (_too_many_expressions, ty) => {
            errors.push(CompileError::MoreThanOneEnumInstantiator {
                span: enum_field_name.span.clone(),
                ty: ty.friendly_type_str(),
            });
            err(warnings, errors)
        }
    }
}
