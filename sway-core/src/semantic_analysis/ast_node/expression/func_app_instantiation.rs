use crate::build_config::BuildConfig;
use crate::control_flow_analysis::ControlFlowGraph;
use crate::error::*;
use crate::semantic_analysis::{ast_node::*, TCOpts, TypeCheckArguments};
use crate::type_engine::TypeId;
use std::cmp::Ordering;

#[allow(clippy::too_many_arguments)]
pub(crate) fn instantiate_function_application(
    decl: TypedFunctionDeclaration,
    name: CallPath,
    type_arguments: Vec<(TypeInfo, Span)>,
    arguments: Vec<Expression>,
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedExpression> {
    let mut warnings = vec![];
    let mut errors = vec![];

    if opts.purity != decl.purity {
        errors.push(CompileError::PureCalledImpure { span: name.span() });
    }

    let arguments_span = arguments
        .iter()
        .map(|x| x.span())
        .reduce(join_spans)
        .unwrap_or_else(|| name.span());

    match arguments.len().cmp(&decl.parameters.len()) {
        Ordering::Greater => {
            errors.push(CompileError::TooManyArgumentsForFunction {
                span: arguments_span,
                method_name: name.suffix.clone(),
                expected: decl.parameters.len(),
                received: arguments.len(),
            });
        }
        Ordering::Less => {
            errors.push(CompileError::TooFewArgumentsForFunction {
                span: arguments_span,
                method_name: name.suffix.clone(),
                expected: decl.parameters.len(),
                received: arguments.len(),
            });
        }
        Ordering::Equal => {}
    }

    // type check arguments in function application vs arguments in function
    // declaration. Use parameter type annotations as annotations for the
    // arguments
    let typed_arguments: Vec<(TypedFunctionParameter, TypedExpression)> = arguments
        .into_iter()
        .zip(decl.parameters.iter())
        .map(|(arg, param)| {
            let args_span = arg.span();
            let args = TypeCheckArguments {
                checkee: arg,
                namespace,
                crate_namespace,
                return_type_annotation: crate::type_engine::insert_type(
                    crate::type_engine::look_up_type_id(param.r#type),
                ),
                help_text: "The argument that has been provided to this function's type does \
                    not match the declared type of the parameter in the function \
                    declaration.",
                self_type,
                build_config,
                dead_code_graph,
                mode: Mode::NonAbi,
                opts,
            };
            let typed_arg = check!(
                TypedExpression::type_check(args),
                error_recovery_expr(args_span),
                warnings,
                errors
            );
            (param.clone(), typed_arg)
        })
        .collect();

    let type_arguments_span = type_arguments
        .iter()
        .map(|(_, span)| span.clone())
        .reduce(join_spans)
        .unwrap_or_else(|| name.span());

    // if this is a generic function, monomorphize its internal types and insert the resulting
    // declaration into the namespace. Then, use that instead.
    let new_decl = match (decl.type_parameters.is_empty(), type_arguments.is_empty()) {
        (true, true) => decl,
        (true, false) => {
            errors.push(CompileError::DoesNotTakeTypeArguments {
                method_name: name.suffix,
                span: type_arguments_span,
            });
            return err(warnings, errors);
        }
        (false, true) => {
            // infer the type arguments from the arguments to the generic function
            let mut type_arguments = vec![];
            for type_parameter in decl.type_parameters.iter() {
                let mut elem = None;
                for (param, arg) in typed_arguments.iter() {
                    let param_type_info = crate::type_engine::look_up_type_id(param.r#type);
                    if type_parameter.name == param_type_info && elem.is_none() {
                        elem = Some((
                            crate::type_engine::look_up_type_id(arg.return_type),
                            arg.span.clone(),
                        ));
                        break;
                    }
                }
                match elem {
                    Some(elem) => {
                        type_arguments.push(elem);
                    }
                    None => {
                        errors.push(CompileError::CannotInferTypeParameter {
                            method_name: name.suffix,
                            param: type_parameter.name_ident.clone(),
                            span: type_parameter.name_ident.span().clone(),
                        });
                        return err(warnings, errors);
                    }
                }
            }
            check!(
                decl.monomorphize(type_arguments, self_type),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
        (false, false) => {
            match type_arguments.len().cmp(&decl.type_parameters.len()) {
                Ordering::Greater => {
                    errors.push(CompileError::TooManyTypeArgumentsForFunction {
                        span: type_arguments_span,
                        method_name: name.suffix.clone(),
                        expected: decl.type_parameters.len(),
                        received: type_arguments.len(),
                    });
                }
                Ordering::Less => {
                    errors.push(CompileError::TooFewTypeArgumentsForFunction {
                        span: type_arguments_span,
                        method_name: name.suffix.clone(),
                        expected: decl.type_parameters.len(),
                        received: type_arguments.len(),
                    });
                }
                Ordering::Equal => {}
            }
            check!(
                decl.monomorphize(type_arguments, self_type),
                return err(warnings, errors),
                warnings,
                errors
            )
        }
    };

    let typed_call_arguments = typed_arguments
        .into_iter()
        .map(|(param, arg)| (param.name, arg))
        .collect::<Vec<_>>();

    ok(
        TypedExpression {
            return_type: new_decl.return_type,
            // now check the function call return type
            // FEATURE this IsConstant can be true if the function itself is
            // constant-able const functions would be an
            // advanced feature and are not supported right
            // now
            is_constant: IsConstant::No,
            expression: TypedExpressionVariant::FunctionApplication {
                arguments: typed_call_arguments,
                name,
                function_body: new_decl.body,
                selector: None, // regular functions cannot be in a contract call; only methods
            },
            span: new_decl.span,
        },
        warnings,
        errors,
    )
}
