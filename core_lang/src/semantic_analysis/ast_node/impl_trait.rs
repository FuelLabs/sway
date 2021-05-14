use super::{declaration::TypedTraitFn, ERROR_RECOVERY_DECLARATION};
use crate::parse_tree::ImplTrait;
use crate::semantic_analysis::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
use crate::{
    error::*,
    types::{MaybeResolvedType, PartiallyResolvedType, ResolvedType},
    Ident,
};

pub(crate) fn implementation_of_trait<'sc>(
    impl_trait: ImplTrait<'sc>,
    namespace: &mut Namespace<'sc>,
) -> CompileResult<'sc, TypedDeclaration<'sc>> {
    let mut errors = vec![];
    let mut warnings = vec![];
    let ImplTrait {
        trait_name,
        type_arguments,
        functions,
        type_implementing_for,
        type_arguments_span,
        block_span,
    } = impl_trait;
    let type_implementing_for = namespace.resolve_type_without_self(&type_implementing_for);
    let self_type = type_implementing_for;
    match namespace.get_call_path(&trait_name) {
        CompileResult::Ok {
            value: TypedDeclaration::TraitDeclaration(tr),
            warnings: mut l_w,
            errors: mut l_e,
        } => {
            errors.append(&mut l_e);
            warnings.append(&mut l_w);
            let tr = tr.clone();
            if type_arguments.len() != tr.type_parameters.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: tr.type_parameters.len(),
                    span: type_arguments_span,
                })
            }

            // type check all components of the impl trait functions
            let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
            // this list keeps track of the remaining functions in the
            // interface surface that still need to be implemented for the
            // trait to be fully implemented
            let mut function_checklist: Vec<&Ident> = tr
                .interface_surface
                .iter()
                .map(|TypedTraitFn { name, .. }| name)
                .collect();
            for fn_decl in functions.into_iter() {
                // replace SelfType with type of implementor
                // i.e. fn add(self, other: u64) -> Self becomes fn
                // add(self: u64, other: u64) -> u64

                let mut fn_decl = type_check!(
                    TypedFunctionDeclaration::type_check(
                        fn_decl.clone(),
                        &namespace,
                        None,
                        "",
                        &self_type
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
                            name: fn_decl.name.primary_name.clone(),
                            trait_name: trait_name.span().as_str(),
                            span: fn_decl.name.span.clone(),
                        });
                        return err(warnings, errors);
                    }
                };
                function_checklist.remove(ix_of_thing_to_remove);

                let mut type_arguments = type_arguments.clone();
                // add generic params from impl trait into function type params
                fn_decl.type_parameters.append(&mut type_arguments);

                // ensure this fn decl's parameters and signature lines up with the one
                // in the trait
                if let Some(mut l_e) = tr.interface_surface.iter().find_map(|TypedTraitFn { name, parameters, return_type, return_type_span }| {
                    if fn_decl.name == *name {
                        let mut errors = vec![];
                        if let Some(mut maybe_err) = parameters.iter().zip(fn_decl.parameters.iter()).find_map(|(fn_decl_param, trait_param)| {
                            let mut errors = vec![];
                            if let MaybeResolvedType::Partial(PartiallyResolvedType::Generic { .. /* TODO use trait constraints as part of the type here to implement trait constraint solver */ }) = fn_decl_param.r#type {
                                match trait_param.r#type {
                                    MaybeResolvedType::Partial(PartiallyResolvedType::Generic { .. }) => (),
                                    _ => 

                                    errors.push(CompileError::MismatchedTypeInTrait {
                                        span: trait_param.type_span.clone(),
                                        given: trait_param.r#type.friendly_type_str(),
                                        expected: fn_decl_param.r#type.friendly_type_str()
                                    })
                                }
                            } else {
                                let fn_decl_param_type = type_check!(
                                    fn_decl_param.r#type.force_resolution(
                                        &self_type,
                                        &fn_decl_param.type_span
                                    ),
                                    return Some(errors),
                                    warnings,
                                    errors
                                );
                                let trait_param_type = type_check!(
                                    trait_param.r#type.force_resolution(
                                        &self_type,
                                        &fn_decl_param.type_span
                                    ),
                                    return Some(errors),
                                    warnings,
                                    errors
                                );

                                if fn_decl_param_type != trait_param_type  {
                                    errors.push(CompileError::MismatchedTypeInTrait {
                                        span: trait_param.type_span.clone(),
                                        given: trait_param.r#type.friendly_type_str(),
                                        expected: fn_decl_param.r#type.friendly_type_str()
                                    });
                                }
                            }
                            if errors.is_empty() { None } else { Some(errors) }
                        }) {
                            errors.append(&mut maybe_err);
                        }
                        let return_type = type_check!(
                            return_type.force_resolution(&self_type, return_type_span),
                            ResolvedType::ErrorRecovery,
                            warnings,
                            errors
                        );
                        if fn_decl.return_type != MaybeResolvedType::Resolved(return_type.clone()) {
                            errors.push(CompileError::MismatchedTypeInTrait {
                                span: fn_decl.return_type_span.clone(),
                                expected: return_type.friendly_type_str(),
                                given: fn_decl.return_type.friendly_type_str() 
                            });
                        }
                        if errors.is_empty() { None } else { Some(errors) }
                    } else {
                        None 
                    } 
                })
                {
                    errors.append(&mut l_e);
                    continue;
                }

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

            namespace.insert_trait_implementation(
                trait_name.clone(),
                self_type,
                functions_buf.clone(),
            );
            ok(
                TypedDeclaration::ImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                },
                warnings,
                errors,
            )
        }
        CompileResult::Ok {
            value: _,
            errors: mut l_e,
            warnings: mut l_w,
        } => {
            errors.append(&mut l_e);
            warnings.append(&mut l_w);
            errors.push(CompileError::NotATrait {
                span: trait_name.span(),
                name: trait_name.suffix.primary_name.clone(),
            });
            ok(ERROR_RECOVERY_DECLARATION.clone(), warnings, errors)
        }
        CompileResult::Err {
            warnings: mut l_w,
            errors: mut l_e,
        } => {
            errors.append(&mut l_e);
            warnings.append(&mut l_w);
            errors.push(CompileError::UnknownTrait {
                name: trait_name.suffix.primary_name,
                span: trait_name.span(),
            });
            ok(ERROR_RECOVERY_DECLARATION.clone(), warnings, errors)
        }
    }
}
