use super::{declaration::TypedTraitFn, ERROR_RECOVERY_DECLARATION};
use crate::parse_tree::{FunctionDeclaration, ImplTrait, TypeParameter};
use crate::semantic_analysis::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
use crate::span::Span;
use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    error::*,
    types::{MaybeResolvedType, PartiallyResolvedType, ResolvedType},
    CallPath, Ident,
};

pub(crate) fn implementation_of_trait<'sc>(
    impl_trait: ImplTrait<'sc>,
    namespace: &mut Namespace<'sc>,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> CompileResult<'sc, TypedDeclaration<'sc>> {
    let mut errors = vec![];
    let mut warnings = vec![];
    let ImplTrait {
        trait_name,
        type_arguments,
        functions,
        type_implementing_for,
        type_implementing_for_span,
        type_arguments_span,
        block_span,
    } = impl_trait;
    if !type_arguments.is_empty() {
        errors.push(CompileError::Internal(
            "Where clauses are not supported yet.",
            type_arguments[0].clone().name_ident.span,
        ));
    }
    let type_implementing_for_id = namespace.insert_type(type_implementing_for);
    match namespace
        .get_call_path(&trait_name)
        .ok(&mut warnings, &mut errors)
    {
        Some(TypedDeclaration::TraitDeclaration(tr)) => {
            let tr = tr.clone();
            if type_arguments.len() != tr.type_parameters.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: tr.type_parameters.len(),
                    span: type_arguments_span,
                })
            }

            let functions_buf = check!(
                type_check_trait_implementation(
                    &tr.interface_surface,
                    &functions,
                    &tr.methods,
                    &tr.name,
                    &tr.type_parameters,
                    namespace,
                    &self_type,
                    build_config,
                    dead_code_graph,
                    &block_span,
                    &type_implementing_for,
                    Mode::NonAbi,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // type check all components of the impl trait functions
            // add the methods to the namespace

            namespace.insert_trait_implementation(
                trait_name.clone(),
                namespace.look_up_type_id(self_type),
                functions_buf.clone(),
            );
            ok(
                TypedDeclaration::ImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    type_implementing_for,
                },
                warnings,
                errors,
            )
        }
        Some(TypedDeclaration::AbiDeclaration(abi)) => {
            // if you are comparing this with the `impl_trait` branch above, note that
            // there are no type arguments here because we don't support generic types
            // in contract ABIs yet (or ever?) due to the complexity of communicating
            // the ABI layout in the descriptor file.
            if type_implementing_for != TypeInfo::Contract {
                errors.push(CompileError::ImplAbiForNonContract {
                    span: type_implementing_for_span.clone(),
                    ty: type_implementing_for.friendly_type_str(),
                });
            }

            let functions_buf = check!(
                type_check_trait_implementation(
                    &abi.interface_surface,
                    &functions,
                    &abi.methods,
                    &abi.name,
                    // ABIs don't have type parameters
                    &[],
                    namespace,
                    &self_type,
                    build_config,
                    dead_code_graph,
                    &block_span,
                    &type_implementing_for,
                    Mode::ImplAbiFn
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // type check all components of the impl trait functions
            // add the methods to the namespace

            namespace.insert_trait_implementation(
                trait_name.clone(),
                namespace.look_up_type_id(self_type),
                functions_buf.clone(),
            );
            ok(
                TypedDeclaration::ImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    type_implementing_for,
                },
                warnings,
                errors,
            )
        }
        Some(_) | None => {
            errors.push(CompileError::UnknownTrait {
                name: trait_name.suffix.primary_name,
                span: trait_name.span(),
            });
            ok(ERROR_RECOVERY_DECLARATION.clone(), warnings, errors)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ImplAbiFn,
    NonAbi,
}

fn type_check_trait_implementation<'sc>(
    interface_surface: &[TypedTraitFn<'sc>],
    functions: &[FunctionDeclaration<'sc>],
    methods: &[FunctionDeclaration<'sc>],
    trait_name: &Ident<'sc>,
    type_arguments: &[TypeParameter<'sc>],
    namespace: &mut Namespace<'sc>,
    self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
    block_span: &Span<'sc>,
    type_implementing_for: TypeInfo<'sc>,
    mode: Mode,
) -> CompileResult<'sc, Vec<TypedFunctionDeclaration<'sc>>> {
    let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    // this list keeps track of the remaining functions in the
    // interface surface that still need to be implemented for the
    // trait to be fully implemented
    let mut function_checklist: Vec<&Ident> = interface_surface
        .iter()
        .map(|TypedTraitFn { name, .. }| name)
        .collect();
    for fn_decl in functions.into_iter() {
        // replace SelfType with type of implementor
        // i.e. fn add(self, other: u64) -> Self becomes fn
        // add(self: u64, other: u64) -> u64

        let mut fn_decl = check!(
            TypedFunctionDeclaration::type_check(
                fn_decl.clone(),
                namespace,
                namespace.insert_type(TypeInfo::Unknown),
                "",
                self_type,
                build_config,
                dead_code_graph,
                mode
            ),
            continue,
            warnings,
            errors
        );
        // remove this function from the "checklist"
        let ix_of_thing_to_remove = match function_checklist
            .iter()
            .position(|name| **name == fn_decl.name)
        {
            Some(ix) => ix,
            None => {
                errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: &(*fn_decl.name.primary_name),
                    trait_name: trait_name.span.as_str().to_string(),
                    span: fn_decl.name.span.clone(),
                });
                return err(warnings, errors);
            }
        };
        function_checklist.remove(ix_of_thing_to_remove);

        let type_arguments = &(*type_arguments);
        // add generic params from impl trait into function type params
        fn_decl.type_parameters.append(&mut type_arguments.to_vec());

        // ensure this fn decl's parameters and signature lines up with the one
        // in the trait
        if let Some(mut l_e) = interface_surface.iter().find_map(
            |TypedTraitFn {
                 name,
                 parameters,
                 return_type,
                 return_type_span,
             }| {
                if fn_decl.name == *name {
                    if fn_decl.parameters.len() != parameters.len() {
                        errors.push(
                            CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                                span: fn_decl.parameters_span(),
                                fn_name: fn_decl.name.primary_name,
                                trait_name: trait_name.primary_name,
                                num_args: parameters.len(),
                                provided_args: fn_decl.parameters.len(),
                            },
                        );
                    }
                    let mut errors = vec![];
                    if let Some(mut maybe_err) = parameters
                        .iter()
                        .zip(fn_decl.parameters.iter())
                        .find_map(|(fn_decl_param, trait_param)| {
                            let mut errors = vec![];
                            // TODO use trait constraints as part of the type here to
                            // implement trait constraint solver */
                            let fn_decl_param_type = fn_decl_param.r#type;
                            let trait_param_type = trait_param.r#type;

                            let real_fn_decl_param_type =
                                namespace.look_up_type_id(fn_decl_param_type);
                            let real_trait_param_type = namespace.look_up_type_id(trait_param_type);
                            if real_fn_decl_param_type != real_trait_param_type {
                                errors.push(CompileError::MismatchedTypeInTrait {
                                    span: trait_param.type_span.clone(),
                                    given: real_trait_param_type.friendly_type_str(),
                                    expected: real_fn_decl_param_type.friendly_type_str(),
                                });
                            }
                            if errors.is_empty() {
                                None
                            } else {
                                Some(errors)
                            }
                        })
                    {
                        errors.append(&mut maybe_err);
                    }
                    let real_ret_type = namespace.look_up_type_id(*return_type);
                    let real_fn_decl_ty = namespace.look_up_type_id(fn_decl.return_type);
                    if real_fn_decl_ty != real_ret_type {
                        errors.push(CompileError::MismatchedTypeInTrait {
                            span: fn_decl.return_type_span.clone(),
                            expected: real_ret_type.friendly_type_str(),
                            given: real_fn_decl_ty.friendly_type_str(),
                        });
                    }
                    if errors.is_empty() {
                        None
                    } else {
                        Some(errors)
                    }
                } else {
                    None
                }
            },
        ) {
            errors.append(&mut l_e);
            continue;
        }

        functions_buf.push(fn_decl);
    }

    // this name space is temporary! It is used only so that the below methods
    // can reference functions from the interface
    let mut local_namespace = namespace.clone();
    local_namespace.insert_trait_implementation(
        CallPath {
            prefixes: vec![],
            suffix: trait_name.clone(),
        },
        type_implementing_for.clone(),
        functions_buf.clone(),
    );
    for method in methods {
        // type check the method now that the interface
        // it depends upon has been implemented

        // use a local namespace which has the above interface inserted
        // into it as a trait implementation for this
        let method = check!(
            TypedFunctionDeclaration::type_check(
                method.clone(),
                &mut local_namespace,
                namespace.insert_type(TypeInfo::Unknown),
                "",
                self_type,
                build_config,
                dead_code_graph,
                mode
            ),
            continue,
            warnings,
            errors
        );
        let self_type_id = namespace.insert_type(type_implementing_for);
        let fn_decl = method.replace_self_types(self_type_id);
        functions_buf.push(fn_decl);
    }

    // check that the implementation checklist is complete
    if !function_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: function_checklist
                .into_iter()
                .map(|Ident { primary_name, .. }| primary_name.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    ok(functions_buf, warnings, errors)
}
