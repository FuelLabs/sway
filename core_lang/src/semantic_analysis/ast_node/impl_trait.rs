use super::{
    declaration::{TypedFunctionParameter, TypedTraitFn},
    ERROR_RECOVERY_DECLARATION,
};
use crate::parse_tree::ImplTrait;
use crate::semantic_analysis::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
use crate::{error::*, types::ResolvedType, Ident};

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
    let type_implementing_for = namespace.resolve_type(&type_implementing_for);
    match namespace.get_symbol(&trait_name) {
        Some(TypedDeclaration::TraitDeclaration(tr)) => {
            let mut tr = tr.clone();
            if type_arguments.len() != tr.type_parameters.len() {
                errors.push(CompileError::IncorrectNumberOfTypeArguments {
                    given: type_arguments.len(),
                    expected: tr.type_parameters.len(),
                    span: type_arguments_span,
                })
            }
            // replace all references to Self in the interface surface with the
            // concrete type
            for TypedTraitFn {
                ref mut parameters,
                ref mut return_type,
                ..
            } in tr.interface_surface.iter_mut()
            {
                parameters.iter_mut().for_each(
                    |TypedFunctionParameter { ref mut r#type, .. }| {
                        if r#type == &ResolvedType::SelfType {
                            *r#type = type_implementing_for.clone();
                        }
                    },
                );
                if return_type == &ResolvedType::SelfType {
                    *return_type = type_implementing_for.clone();
                }
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
                        Some(type_implementing_for.clone())
                    ),
                    continue,
                    warnings,
                    errors
                );
                /*
                fn_decl
                    .parameters
                    .iter_mut()
                    .filter(|TypedFunctionParameter { r#type, .. }| r#type == &ResolvedType::SelfType)
                    .for_each(|TypedFunctionParameter { ref mut r#type, .. }| {
                        *r#type = type_implementing_for.clone()
                    });
                */

                if fn_decl.return_type == ResolvedType::SelfType {
                    fn_decl.return_type = type_implementing_for.clone();
                }

                let mut type_arguments = type_arguments.clone();
                // add generic params from impl trait into function type params
                fn_decl.type_parameters.append(&mut type_arguments);
                // TODO handle these generics smartly
                // replace all references to type_implementing_for with Self

                // ensure this fn decl's parameters and signature lines up with the one
                // in the trait
                if let Some(mut l_e) = tr.interface_surface.iter().find_map(|TypedTraitFn { name, parameters, return_type }| {
                    if fn_decl.name == *name {
                        let mut errors = vec![];
                        if let Some(mut maybe_err) = parameters.iter().zip(fn_decl.parameters.iter()).find_map(|(fn_decl_param, trait_param)| {
                            let mut errors = vec![];
                            if let ResolvedType::Generic { .. /* TODO use trait constraints as part of the type here to implement trait constraint solver */ } = fn_decl_param.r#type {
                                match trait_param.r#type {
                                    ResolvedType::Generic { .. } => (),
                                    _ => 

                                    errors.push(CompileError::MismatchedTypeInTrait {
                                        span: trait_param.type_span.clone(),
                                        given: trait_param.r#type.friendly_type_str(),
                                        expected: fn_decl_param.r#type.friendly_type_str()
                                    })
                                }
                            } else {
                                if fn_decl_param.r#type != trait_param.r#type  {
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
                        if fn_decl.return_type != *return_type {
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
                // remove this function from the "checklist"
                let ix_of_thing_to_remove = match function_checklist
                    .iter()
                    .position(|name| **name == fn_decl.name)
                {
                    Some(ix) => ix,
                    None => {
                        errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                            name: fn_decl.name.primary_name.clone(),
                            trait_name: trait_name.primary_name.clone(),
                            span: fn_decl.name.span.clone(),
                        });
                        return err(warnings, errors);
                    }
                };
                function_checklist.remove(ix_of_thing_to_remove);

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
                type_implementing_for,
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
        Some(_) => {
            errors.push(CompileError::NotATrait {
                span: trait_name.span,
                name: trait_name.primary_name,
            });
            ok(ERROR_RECOVERY_DECLARATION.clone(), warnings, errors)
        }
        None => {
            errors.push(CompileError::UnknownTrait {
                name: trait_name.primary_name,
                span: trait_name.span,
            });
            ok(ERROR_RECOVERY_DECLARATION.clone(), warnings, errors)
        }
    }
}
