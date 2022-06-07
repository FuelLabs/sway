use sway_types::{Ident, Span, Spanned};

use crate::{
    error::{err, ok},
    semantic_analysis::{Mode, TCOpts, TypeCheckArguments},
    type_engine::{
        insert_type, look_up_type_id, resolve_type, unify_with_self, CopyTypes, TypeId, TypeMapping, insert_type_parameters, UpdateTypes,
    },
    CallPath, CompileError, CompileResult, FunctionDeclaration, ImplSelf,
    ImplTrait, Namespace, Purity, TypeInfo, TypedDeclaration, TypedFunctionDeclaration,
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
    ) -> CompileResult<TypedImplTrait> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let ImplTrait {
            trait_name,
            type_arguments,
            functions,
            type_implementing_for,
            type_implementing_for_span,
            block_span,
        } = impl_trait;

        let implementing_for_type_id = check!(
            namespace.resolve_type_without_self(type_implementing_for),
            return err(warnings, errors),
            warnings,
            errors
        );
        for type_argument in type_arguments.iter() {
            if !type_argument.trait_constraints.is_empty() {
                errors.push(CompileError::WhereClauseNotYetSupported {
                    span: type_argument.name_ident.span(),
                });
                break;
            }
        }
        let impl_trait = match namespace
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
                        namespace,
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

                namespace.insert_trait_implementation(
                    trait_name.clone(),
                    match resolve_type(implementing_for_type_id, &type_implementing_for_span) {
                        Ok(o) => o,
                        Err(e) => {
                            errors.push(e.into());
                            return err(warnings, errors);
                        }
                    },
                    functions_buf.clone(),
                );

                TypedImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    implementing_for_type_id,
                }
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
                        namespace,
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

                namespace.insert_trait_implementation(
                    trait_name.clone(),
                    look_up_type_id(implementing_for_type_id),
                    functions_buf.clone(),
                );
                TypedImplTrait {
                    trait_name,
                    span: block_span,
                    methods: functions_buf,
                    implementing_for_type_id,
                }
            }
            Some(_) | None => {
                errors.push(CompileError::UnknownTrait {
                    name: trait_name.suffix.clone(),
                    span: trait_name.span(),
                });
                return err(warnings, errors);
            }
        };
        ok(impl_trait, warnings, errors)
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

        // insert type parameters as Unknown types
        let type_mapping = insert_type_parameters(&type_parameters);

        // update the types in the type parameters, insert the type parameters
        // into the decl namespace, and check to see if the type parameters
        // shadow one another
        for type_parameter in type_parameters.iter_mut() {
            check!(
                type_parameter.update_types_without_self(&type_mapping, &mut namespace),
                return err(warnings, errors),
                warnings,
                errors
            );
            let type_parameter_decl = TypedDeclaration::GenericTypeForFunctionScope {
                name: type_parameter.name_ident.clone(),
                type_id: type_parameter.type_id,
            };
            check!(
                namespace.insert_symbol(type_parameter.name_ident.clone(), type_parameter_decl),
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
    let mut functions_buf: Vec<TypedFunctionDeclaration> = vec![];
    let mut processed_fns = std::collections::HashSet::<Ident>::new();
    let mut errors = vec![];
    let mut warnings = vec![];
    let self_type_id = type_implementing_for;
    // this map keeps track of the remaining functions in the
    // interface surface that still need to be implemented for the
    // trait to be fully implemented
    let mut function_checklist: std::collections::BTreeMap<&Ident, _> = interface_surface
        .iter()
        .map(|decl| (&decl.name, decl))
        .collect();
    for fn_decl in functions {
        // replace SelfType with type of implementor
        // i.e. fn add(self, other: u64) -> Self becomes fn
        // add(self: u64, other: u64) -> u64

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
        let fn_decl = fn_decl.replace_self_types(self_type_id);

        // Ensure that there aren't multiple definitions of this function impl'd.
        if !processed_fns.insert(fn_decl.name.clone()) {
            errors.push(CompileError::MultipleDefinitionsOfFunction {
                name: fn_decl.name.clone(),
            });
            return err(warnings, errors);
        }
        // remove this function from the "checklist"
        let trait_fn = match function_checklist.remove(&fn_decl.name) {
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

        // ensure this fn decl's parameters and signature lines up with the one
        // in the trait
        let TypedTraitFn {
            name: _,
            purity,
            parameters,
            return_type,
            return_type_span: _,
        } = trait_fn;

        if fn_decl.parameters.len() != parameters.len() {
            errors.push(
                CompileError::IncorrectNumberOfInterfaceSurfaceFunctionParameters {
                    span: fn_decl.parameters_span(),
                    fn_name: fn_decl.name.clone(),
                    trait_name: trait_name.suffix.clone(),
                    num_args: parameters.len(),
                    provided_args: fn_decl.parameters.len(),
                },
            );
        }

        for (trait_param, fn_decl_param) in parameters.iter().zip(&fn_decl.parameters) {
            // TODO use trait constraints as part of the type here to
            // implement trait constraint solver */
            let fn_decl_param_type = fn_decl_param.r#type;
            let trait_param_type = trait_param.r#type;

            let (mut new_warnings, new_errors) = unify_with_self(
                fn_decl_param_type,
                trait_param_type,
                self_type_id,
                &trait_param.type_span,
                "",
            );

            warnings.append(&mut new_warnings);
            if !new_errors.is_empty() {
                errors.push(CompileError::MismatchedTypeInTrait {
                    span: fn_decl_param.type_span.clone(),
                    given: fn_decl_param_type.to_string(),
                    expected: trait_param_type.to_string(),
                });
                break;
            }
        }

        if fn_decl.purity != *purity {
            errors.push(if *purity == Purity::Pure {
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
                    attrs: purity.to_attribute_syntax(),
                    span: fn_decl.span.clone(),
                }
            });
        }

        let (mut new_warnings, new_errors) = unify_with_self(
            *return_type,
            fn_decl.return_type,
            self_type_id,
            &fn_decl.return_type_span,
            "",
        );
        warnings.append(&mut new_warnings);
        if !new_errors.is_empty() {
            errors.push(CompileError::MismatchedTypeInTrait {
                span: fn_decl.return_type_span.clone(),
                expected: return_type.to_string(),
                given: fn_decl.return_type.to_string(),
            });

            continue;
        }

        functions_buf.push(fn_decl);
    }

    // This name space is temporary! It is used only so that the below methods
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
        let method = check!(
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
                .map(|(ident, _)| ident.as_str().to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    ok(functions_buf, warnings, errors)
}
