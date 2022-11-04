use std::collections::BTreeMap;

use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{style::is_upper_camel_case, Ident, Spanned};

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed::*, ty, CallPath},
    semantic_analysis::{Mode, TypeCheckContext},
    type_system::*,
};

impl ty::TyTraitDeclaration {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        trait_decl: TraitDeclaration,
    ) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let TraitDeclaration {
            name,
            type_parameters,
            attributes,
            interface_surface,
            methods,
            supertraits,
            visibility,
            span,
        } = trait_decl;

        if !is_upper_camel_case(name.as_str()) {
            warnings.push(CompileWarning {
                span: name.span(),
                warning_content: Warning::NonClassCaseTraitName { name: name.clone() },
            })
        }

        // A temporary namespace for checking within the trait's scope.
        let self_type = insert_type(TypeInfo::SelfType);
        let mut trait_namespace = ctx.namespace.clone();
        let mut ctx = ctx.scoped(&mut trait_namespace).with_self_type(self_type);

        // type check the type parameters, which will insert them into the namespace
        let mut new_type_parameters = vec![];
        for type_parameter in type_parameters.into_iter() {
            new_type_parameters.push(check!(
                TypeParameter::type_check(ctx.by_ref(), type_parameter),
                return err(warnings, errors),
                warnings,
                errors
            ));
        }

        // Recursively handle supertraits: make their interfaces and methods available to this trait
        check!(
            handle_supertraits(ctx.by_ref(), &supertraits),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the interface surface
        let mut new_interface_surface = vec![];
        let mut dummy_interface_surface = vec![];
        for method in interface_surface.into_iter() {
            let method = check!(
                ty::TyTraitFn::type_check(ctx.by_ref(), method),
                return err(warnings, errors),
                warnings,
                errors
            );
            let decl_id = de_insert_trait_fn(method.clone());
            new_interface_surface.push(decl_id.clone());
            dummy_interface_surface
                .push(de_insert_function(method.to_dummy_func(Mode::NonAbi)).with_parent(decl_id));
        }

        // insert placeholder functions representing the interface surface
        // to allow methods to use those functions
        check!(
            ctx.namespace.insert_trait_implementation(
                CallPath {
                    prefixes: vec![],
                    suffix: name.clone(),
                    is_absolute: false,
                },
                new_type_parameters.iter().map(|x| x.into()).collect(),
                self_type,
                &dummy_interface_surface,
                &span,
                false
            ),
            return err(warnings, errors),
            warnings,
            errors
        );

        // type check the methods
        let mut new_methods = vec![];
        for method in methods.into_iter() {
            let method = check!(
                ty::TyFunctionDeclaration::type_check(ctx.by_ref(), method.clone(), true),
                ty::TyFunctionDeclaration::error(method),
                warnings,
                errors
            );
            new_methods.push(de_insert_function(method));
        }

        let typed_trait_decl = ty::TyTraitDeclaration {
            name,
            type_parameters: new_type_parameters,
            interface_surface: new_interface_surface,
            methods: new_methods,
            supertraits,
            visibility,
            attributes,
            span,
        };
        ok(typed_trait_decl, warnings, errors)
    }

    /// Retrieves the interface surface and implemented methods for this trait.
    pub(crate) fn retrieve_interface_surface_and_implemented_methods_for_type(
        &self,
        ctx: TypeCheckContext,
        type_id: TypeId,
        call_path: &CallPath,
    ) -> CompileResult<(
        BTreeMap<Ident, DeclarationId>,
        BTreeMap<Ident, DeclarationId>,
    )> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let mut interface_surface_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();
        let mut impld_method_ids: BTreeMap<Ident, DeclarationId> = BTreeMap::new();

        let ty::TyTraitDeclaration {
            interface_surface,
            name,
            ..
        } = self;

        // Retrieve the interface surface for this trait.
        for decl_id in interface_surface.iter() {
            let method = check!(
                CompileResult::from(de_get_trait_fn(decl_id.clone(), &call_path.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            interface_surface_method_ids.insert(method.name, decl_id.clone());
        }

        // Retrieve the implemented methods for the interface surface for the
        // supertrait and this type.
        for decl_id in ctx
            .namespace
            .get_methods_for_type_and_trait_name(type_id, call_path)
            .into_iter()
        {
            let method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &name.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            impld_method_ids.insert(method.name, decl_id);
        }

        ok(
            (interface_surface_method_ids, impld_method_ids),
            warnings,
            errors,
        )
    }
}

/// Recursively handle supertraits by adding all their interfaces and methods to some namespace
/// which is meant to be the namespace of the subtrait in question
fn handle_supertraits(mut ctx: TypeCheckContext, supertraits: &[Supertrait]) -> CompileResult<()> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let self_type = ctx.self_type();

    for supertrait in supertraits.iter() {
        // Right now we don't have the ability to support defining a supertrait
        // using a callpath directly, so we check to see if the user has done
        // this and we disallow it.
        if !supertrait.name.prefixes.is_empty() {
            errors.push(CompileError::UnimplementedWithHelp(
                "Using module paths to define supertraits is not supported yet.",
                "try importing the trait with a \"use\" statement instead",
                supertrait.span(),
            ));
            continue;
        }

        match ctx
            .namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDeclaration::TraitDeclaration(decl_id)) => {
                let ty::TyTraitDeclaration {
                    interface_surface,
                    methods,
                    supertraits,
                    name,
                    type_parameters,
                    ..
                } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
                    break,
                    warnings,
                    errors
                );

                // Right now we don't parse type arguments for supertraits, so
                // we should give this error message to users.
                if !type_parameters.is_empty() {
                    errors.push(CompileError::Unimplemented(
                        "Using generic traits as supertraits is not supported yet.",
                        supertrait.name.span(),
                    ));
                    continue;
                }

                let mut all_methods = methods;

                // Retrieve the interface surface for this trait.
                for decl_id in interface_surface.into_iter() {
                    let mut method = check!(
                        CompileResult::from(de_get_trait_fn(decl_id.clone(), &name.span())),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                    method.replace_self_type(self_type);
                    all_methods.push(
                        de_insert_function(method.to_dummy_func(Mode::NonAbi)).with_parent(decl_id),
                    );
                }

                // Insert the methods of the supertrait into the namespace.
                // Specifically do not check for conflicting definitions because
                // this is just a temporary namespace for type checking and
                // these are not actual impl blocks.
                ctx.namespace.insert_trait_implementation(
                    supertrait.name.clone(),
                    type_parameters.iter().map(|x| x.into()).collect(),
                    self_type,
                    &all_methods,
                    &supertrait.name.span(),
                    false,
                );

                // Recurse to insert versions of interfaces and methods of the
                // *super* supertraits.
                check!(
                    handle_supertraits(ctx.by_ref(), &supertraits),
                    break,
                    warnings,
                    errors
                );
            }
            Some(ty::TyDeclaration::AbiDeclaration(_)) => {
                errors.push(CompileError::AbiAsSupertrait {
                    span: supertrait.name.span().clone(),
                })
            }
            _ => errors.push(CompileError::TraitNotFound {
                name: supertrait.name.to_string(),
                span: supertrait.name.span(),
            }),
        }
    }

    if errors.is_empty() {
        ok((), warnings, errors)
    } else {
        err(warnings, errors)
    }
}
