use super::{declaration::TypedTraitFn, ERROR_RECOVERY_DECLARATION};

use crate::{
    build_config::BuildConfig,
    control_flow_analysis::ControlFlowGraph,
    error::*,
    parse_tree::{FunctionDeclaration, ImplTrait, TypeParameter},
    semantic_analysis::*,
    type_engine::{
        insert_type, look_up_type_id, resolve_type, FriendlyTypeString, TypeId, TypeInfo,
    },
    CallPath, Ident,
};

use sway_types::span::Span;

pub(crate) fn implementation_of_trait(
    impl_trait: ImplTrait,
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    opts: TCOpts,
) -> CompileResult<TypedDeclaration> {
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
    let type_implementing_for = namespace.resolve_type_without_self(&type_implementing_for);
    let type_implementing_for = look_up_type_id(type_implementing_for);
    let type_implementing_for_id = insert_type(type_implementing_for.clone());
    if !type_arguments.is_empty() {
        errors.push(CompileError::Internal(
            "Where clauses are not supported yet.",
            type_arguments[0].clone().name_ident.span().clone(),
        ));
    }
    match namespace
        .get_call_path(&trait_name)
        .ok(&mut warnings, &mut errors)
    {
        Some(TypedDeclaration::TraitDeclaration(tr)) => {
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
                    crate_namespace,
                    type_implementing_for_id,
                    build_config,
                    dead_code_graph,
                    &block_span,
                    type_implementing_for_id,
                    &type_implementing_for_span,
                    Mode::NonAbi,
                    opts,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // type check all components of the impl trait functions
            // add the methods to the namespace

            namespace.insert_trait_implementation(
                trait_name.clone(),
                match resolve_type(type_implementing_for_id, &type_implementing_for_span) {
                    Ok(o) => o,
                    Err(e) => {
                        errors.push(e.into());
                        return err(warnings, errors);
                    }
                },
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
                    crate_namespace,
                    type_implementing_for_id,
                    build_config,
                    dead_code_graph,
                    &block_span,
                    type_implementing_for_id,
                    &type_implementing_for_span,
                    Mode::ImplAbiFn,
                    opts,
                ),
                return err(warnings, errors),
                warnings,
                errors
            );
            // type check all components of the impl trait functions
            // add the methods to the namespace

            namespace.insert_trait_implementation(
                trait_name.clone(),
                look_up_type_id(type_implementing_for_id),
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
                name: trait_name.suffix.clone(),
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

impl Default for Mode {
    fn default() -> Self {
        Mode::NonAbi
    }
}

#[allow(clippy::too_many_arguments)]
fn type_check_trait_implementation(
    interface_surface: &[TypedTraitFn],
    functions: &[FunctionDeclaration],
    methods: &[FunctionDeclaration],
    trait_name: &Ident,
    type_arguments: &[TypeParameter],
    namespace: crate::semantic_analysis::NamespaceRef,
    crate_namespace: NamespaceRef,
    _self_type: TypeId,
    build_config: &BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    block_span: &Span,
    type_implementing_for: TypeId,
    type_implementing_for_span: &Span,
    mode: Mode,
    opts: TCOpts,
) -> CompileResult<Vec<TypedFunctionDeclaration>> {
    let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
    let mut errors = vec![];
    let mut warnings = vec![];
    let self_type_id = type_implementing_for;
    // this list keeps track of the remaining functions in the
    // interface surface that still need to be implemented for the
    // trait to be fully implemented
    let mut function_checklist: Vec<&Ident> = interface_surface
        .iter()
        .map(|TypedTraitFn { name, .. }| name)
        .collect();
    for fn_decl in functions {
        // replace SelfType with type of implementor
        // i.e. fn add(self, other: u64) -> Self becomes fn
        // add(self: u64, other: u64) -> u64

        let fn_decl = check!(
            TypedFunctionDeclaration::type_check(TypeCheckArguments {
                checkee: fn_decl.clone(),
                namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type: type_implementing_for,
                build_config,
                dead_code_graph,
                mode,
                opts,
            }),
            continue,
            warnings,
            errors
        );
        let mut fn_decl = fn_decl.replace_self_types(self_type_id);
        // remove this function from the "checklist"
        let ix_of_thing_to_remove = match function_checklist
            .iter()
            .position(|name| **name == fn_decl.name)
        {
            Some(ix) => ix,
            None => {
                errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: fn_decl.name.clone(),
                    trait_name: trait_name.clone(),
                    span: fn_decl.name.span().clone(),
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
                 return_type_span: _,
             }| {
                if fn_decl.name == *name {
                    if fn_decl.parameters.len() != parameters.len() {
                        errors.push(
                            CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                                span: fn_decl.parameters_span(),
                                fn_name: fn_decl.name.clone(),
                                trait_name: trait_name.clone(),
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

                            match crate::type_engine::unify_with_self(
                                fn_decl_param_type,
                                trait_param_type,
                                self_type_id,
                                &trait_param.type_span,
                            ) {
                                Ok(mut ws) => {
                                    warnings.append(&mut ws);
                                }
                                Err(_e) => {
                                    errors.push(CompileError::MismatchedTypeInTrait {
                                        span: trait_param.type_span.clone(),
                                        given: fn_decl_param_type.friendly_type_str(),
                                        expected: trait_param_type.friendly_type_str(),
                                    });
                                }
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

                    match crate::type_engine::unify_with_self(
                        *return_type,
                        fn_decl.return_type,
                        self_type_id,
                        &fn_decl.return_type_span,
                    ) {
                        Ok(mut ws) => {
                            warnings.append(&mut ws);
                        }
                        Err(_e) => {
                            errors.push(CompileError::MismatchedTypeInTrait {
                                span: fn_decl.return_type_span.clone(),
                                expected: return_type.friendly_type_str(),
                                given: fn_decl.return_type.friendly_type_str(),
                            });
                        }
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
    let local_namespace: NamespaceRef = create_new_scope(namespace);
    local_namespace.insert_trait_implementation(
        CallPath {
            prefixes: vec![],
            suffix: trait_name.clone(),
            is_absolute: false,
        },
        match resolve_type(type_implementing_for, type_implementing_for_span) {
            Ok(o) => o,
            Err(e) => {
                errors.push(e.into());
                return err(warnings, errors);
            }
        },
        functions_buf.clone(),
    );
    for method in methods {
        // type check the method now that the interface
        // it depends upon has been implemented

        // use a local namespace which has the above interface inserted
        // into it as a trait implementation for this
        let method = check!(
            TypedFunctionDeclaration::type_check(TypeCheckArguments {
                checkee: method.clone(),
                namespace: local_namespace,
                crate_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: "",
                self_type: type_implementing_for,
                build_config,
                dead_code_graph,
                mode,
                opts,
            }),
            continue,
            warnings,
            errors
        );
        let fn_decl = method.replace_self_types(self_type_id);
        functions_buf.push(fn_decl);
    }

    // check that the implementation checklist is complete
    if !function_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: function_checklist
                .into_iter()
                .map(|ident| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    ok(functions_buf, warnings, errors)
}
