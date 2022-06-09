use std::collections::HashSet;

use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::{Mode, TCOpts, TypeCheckArguments},
    type_engine::{
        insert_type, look_up_type_id, resolve_type, unify_with_self, CopyTypes, TypeId, TypeMapping,
    },
    CallPath, CompileError, CompileResult, FunctionDeclaration, ImplSelf, ImplTrait, Namespace,
    Purity, TypeInfo, TypeParameter, TypedDeclaration, TypedFunctionDeclaration,
};

use super::TypedTraitFn;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedImplTrait {
    pub trait_name: CallPath,
    pub(crate) implementing_for_type_id: TypeId,
    pub methods: Vec<TypedFunctionDeclaration>,
    pub(crate) span: Span,
}

impl CopyTypes for TypedImplTrait {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.methods
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
    }
}

impl TypedImplTrait {
    pub(crate) fn type_check_impl_trait(
        impl_trait: ImplTrait,
        namespace: &mut Namespace,
        opts: TCOpts,
    ) -> CompileResult<(TypedImplTrait, TypeInfo)> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let ImplTrait {
            trait_name,
            mut type_parameters,
            functions,
            type_implementing_for,
            type_implementing_for_span,
            block_span,
        } = impl_trait;

        // if this trait uses a where clause then we do not support it yet
        for type_argument in type_parameters.iter() {
            if !type_argument.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_argument.name_ident.span(),
                });
                break;
            }
        }
        if !errors.is_empty() {
            return err(warnings, errors);
        }

        // create the namespace for the impl
        let mut namespace = namespace.clone();

        // update the types in the type parameters, insert the type parameters
        // into the decl namespace, and check to see if the type parameters
        // shadow one another
        for type_parameter in type_parameters.iter_mut() {
            check!(
                TypeParameter::type_check(type_parameter, &mut namespace),
                continue,
                warnings,
                errors
            );
        }

        // type check the type that we are implementing for
        let implementing_for_type_id = check!(
            namespace.resolve_type_without_self(type_implementing_for),
            return err(warnings, errors),
            warnings,
            errors
        );

        let (impl_trait, type_implementing_for) = match namespace
            .resolve_call_path(&trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(TypedDeclaration::TraitDeclaration(tr)) => {
                let functions_buf = check!(
                    type_check_trait_implementation(
                        &tr.interface_surface,
                        &functions,
                        &tr.methods,
                        &trait_name,
                        &mut namespace,
                        implementing_for_type_id,
                        &block_span,
                        implementing_for_type_id,
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

                let impl_trait = TypedImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    implementing_for_type_id,
                };
                let type_implementing_for =
                    match resolve_type(implementing_for_type_id, &type_implementing_for_span) {
                        Ok(o) => o,
                        Err(e) => {
                            errors.push(e.into());
                            return err(warnings, errors);
                        }
                    };
                (impl_trait, type_implementing_for)
            }
            Some(TypedDeclaration::AbiDeclaration(abi)) => {
                // if you are comparing this with the `impl_trait` branch above, note that
                // there are no type arguments here because we don't support generic types
                // in contract ABIs yet (or ever?) due to the complexity of communicating
                // the ABI layout in the descriptor file.
                if look_up_type_id(implementing_for_type_id) != TypeInfo::Contract {
                    errors.push(CompileError::ImplAbiForNonContract {
                        span: type_implementing_for_span.clone(),
                        ty: implementing_for_type_id.to_string(),
                    });
                }

                let functions_buf = check!(
                    type_check_trait_implementation(
                        &abi.interface_surface,
                        &functions,
                        &abi.methods,
                        &trait_name,
                        &mut namespace,
                        implementing_for_type_id,
                        &block_span,
                        implementing_for_type_id,
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

                let impl_trait = TypedImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    implementing_for_type_id,
                };
                (impl_trait, look_up_type_id(implementing_for_type_id))
            }
            Some(_) | None => {
                errors.push(CompileError::UnknownTrait {
                    name: trait_name.suffix.clone(),
                    span: trait_name.span(),
                });
                return err(warnings, errors);
            }
        };
        ok((impl_trait, type_implementing_for), warnings, errors)
    }

    pub(crate) fn type_check_impl_self(
        impl_self: ImplSelf,
        namespace: &mut Namespace,
        opts: TCOpts,
    ) -> CompileResult<TypedImplTrait> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let ImplSelf {
            type_implementing_for,
            mut type_parameters,
            functions,
            block_span,
            ..
        } = impl_self;

        // create the namespace for the impl
        let mut namespace = namespace.clone();

        // update the types in the type parameters, insert the type parameters
        // into the decl namespace, and check to see if the type parameters
        // shadow one another
        for type_parameter in type_parameters.iter_mut() {
            check!(
                TypeParameter::type_check(type_parameter, &mut namespace),
                continue,
                warnings,
                errors
            );
        }

        // type check the type we are implementing for
        let implementing_for_type_id = check!(
            namespace.resolve_type_without_self(type_implementing_for),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the functions
        let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
        for fn_decl in functions.into_iter() {
            functions_buf.push(check!(
                TypedFunctionDeclaration::type_check(TypeCheckArguments {
                    checkee: fn_decl,
                    namespace: &mut namespace,
                    return_type_annotation: insert_type(TypeInfo::Unknown),
                    help_text: "",
                    self_type: implementing_for_type_id,
                    mode: Mode::NonAbi,
                    opts,
                }),
                continue,
                warnings,
                errors
            ));
        }

        // create the trait name
        let trait_name = CallPath {
            prefixes: vec![],
            suffix: Ident::new_with_override("r#Self", block_span.clone()),
            is_absolute: false,
        };

        let impl_trait = TypedImplTrait {
            trait_name,
            span: block_span,
            methods: functions_buf,
            implementing_for_type_id,
        };
        ok(impl_trait, warnings, errors)
    }
}

#[allow(clippy::too_many_arguments)]
fn type_check_trait_implementation(
    interface_surface: &[TypedTraitFn],
    functions: &[FunctionDeclaration],
    methods: &[FunctionDeclaration],
    trait_name: &CallPath,
    namespace: &mut Namespace,
    _self_type: TypeId,
    block_span: &Span,
    type_implementing_for: TypeId,
    type_implementing_for_span: &Span,
    mode: Mode,
    opts: TCOpts,
) -> CompileResult<Vec<TypedFunctionDeclaration>> {
    let mut errors = vec![];
    let mut warnings = vec![];

    let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
    let mut processed_fns = HashSet::<Ident>::new();
    // this map keeps track of the remaining functions in the
    // interface surface that still need to be implemented for the
    // trait to be fully implemented
    let mut function_checklist: std::collections::BTreeMap<&Ident, _> = interface_surface
        .iter()
        .map(|decl| (&decl.name, decl))
        .collect();

    for fn_decl in functions {
        // type check the function
        let fn_decl = check!(
            TypedFunctionDeclaration::type_check(TypeCheckArguments {
                checkee: fn_decl.clone(),
                namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type: type_implementing_for,
                mode,
                opts,
            }),
            continue,
            warnings,
            errors
        );

        // Ensure that there aren't multiple definitions of this function impl'd.
        if !processed_fns.insert(fn_decl.name.clone()) {
            errors.push(CompileError::MultipleDefinitionsOfFunction {
                name: fn_decl.name.clone(),
            });
            return err(warnings, errors);
        }

        // remove this function from the "checklist" and retrieve the function
        // signature from the trait definition
        let fn_signature = match function_checklist.remove(&fn_decl.name) {
            Some(trait_fn) => trait_fn,
            None => {
                errors.push(CompileError::FunctionNotAPartOfInterfaceSurface {
                    name: fn_decl.name.clone(),
                    trait_name: trait_name.suffix.clone(),
                    span: fn_decl.name.span(),
                });
                return err(warnings, errors);
            }
        };

        // check to see if the number of parameters in the function signature
        // is equal to the number of parameters in the function declaration
        if fn_decl.parameters.len() != fn_signature.parameters.len() {
            errors.push(
                CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                    span: fn_decl.parameters_span(),
                    fn_name: fn_decl.name.clone(),
                    trait_name: trait_name.suffix.clone(),
                    num_args: fn_signature.parameters.len(),
                    provided_args: fn_decl.parameters.len(),
                },
            );
            continue;
        }

        // unify the types of the corresponding elements between the
        // parameters of the function signature and the parameters of the
        // function declaration
        for (fn_signature_param, fn_decl_param) in
            fn_signature.parameters.iter().zip(&fn_decl.parameters)
        {
            // TODO use trait constraints as part of the type here to
            // implement trait constraint solver */
            let (mut new_warnings, new_errors) = unify_with_self(
                fn_decl_param.r#type,
                fn_signature_param.r#type,
                type_implementing_for,
                &fn_signature_param.type_span,
                "",
            );
            warnings.append(&mut new_warnings);
            if !new_errors.is_empty() {
                errors.push(CompileError::MismatchedTypeInTrait {
                    span: fn_decl_param.type_span.clone(),
                    given: fn_decl_param.r#type.to_string(),
                    expected: fn_signature_param.r#type.to_string(),
                });
                break;
            }
        }

        // check to see if the purity of the functions signature is the
        // same as the purity of the function declaration
        if fn_decl.purity != fn_signature.purity {
            errors.push(if fn_signature.purity == Purity::Pure {
                CompileError::TraitDeclPureImplImpure {
                    fn_name: fn_decl.name.clone(),
                    trait_name: trait_name.suffix.clone(),
                    attrs: fn_decl.purity.to_attribute_syntax(),
                    span: fn_decl.span.clone(),
                }
            } else {
                CompileError::TraitImplPurityMismatch {
                    fn_name: fn_decl.name.clone(),
                    trait_name: trait_name.suffix.clone(),
                    attrs: fn_signature.purity.to_attribute_syntax(),
                    span: fn_decl.span.clone(),
                }
            });
        }

        // unify the return type of the function signature and of the
        // function declaration
        let (mut new_warnings, new_errors) = unify_with_self(
            fn_signature.return_type,
            fn_decl.return_type,
            type_implementing_for,
            &fn_decl.return_type_span,
            "",
        );
        warnings.append(&mut new_warnings);
        if !new_errors.is_empty() {
            errors.push(CompileError::MismatchedTypeInTrait {
                span: fn_decl.return_type_span.clone(),
                expected: fn_signature.return_type.to_string(),
                given: fn_decl.return_type.to_string(),
            });
            continue;
        }

        functions_buf.push(fn_decl);
    }

    // This namespace is temporary! It is used only so that the below methods
    // can reference functions from the interface
    let mut impl_trait_namespace = namespace.clone();

    // A trait impl needs access to everything that the trait methods have access to, which is
    // basically everything in the path where the trait is declared.
    // First, get the path to where the trait is declared. This is a combination of the path stored
    // in the symbols map and the path stored in the CallPath.
    let trait_path = [
        &trait_name.prefixes[..],
        impl_trait_namespace.get_canonical_path(&trait_name.suffix),
    ]
    .concat();
    impl_trait_namespace.star_import(&trait_path);

    impl_trait_namespace.insert_trait_implementation(
        CallPath {
            prefixes: vec![],
            suffix: trait_name.suffix.clone(),
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
        functions_buf.push(check!(
            TypedFunctionDeclaration::type_check(TypeCheckArguments {
                checkee: method.clone(),
                namespace: &mut impl_trait_namespace,
                return_type_annotation: insert_type(TypeInfo::Unknown),
                help_text: Default::default(),
                self_type: type_implementing_for,
                mode,
                opts,
            }),
            continue,
            warnings,
            errors
        ));
    }

    // check that the implementation checklist is complete
    if !function_checklist.is_empty() {
        errors.push(CompileError::MissingInterfaceSurfaceMethods {
            span: block_span.clone(),
            missing_functions: function_checklist
                .into_iter()
                .map(|(ident, _)| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
        return err(warnings, errors);
    }

    ok(functions_buf, warnings, errors)
}
