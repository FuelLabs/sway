use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    error::*,
    language::{parsed, ty},
    semantic_analysis::TypeCheckContext,
    EnforceTypeArguments, TypeId,
};

#[derive(Clone, PartialEq, Eq)]
pub enum SupertraitOf {
    Abi(Span), // Span is needed for error reporting
    Trait,
}

/// Recursively insert the interface surfaces and methods from supertraits to
/// the given namespace.
pub(crate) fn insert_supertraits_into_namespace(
    mut ctx: TypeCheckContext,
    type_id: TypeId,
    supertraits: &[parsed::Supertrait],
    supertraits_of: &SupertraitOf,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    let decl_engine = ctx.engines.de();

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

        let decl = ctx
            .namespace
            .resolve_call_path(&supertrait.name)
            .ok(&mut warnings, &mut errors)
            .cloned();

        match (decl.clone(), supertraits_of) {
            // a trait can be a supertrait of either a trait or a an ABI
            (Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })), _) => {
                let mut trait_decl = decl_engine.get_trait(&decl_id);

                // Right now we don't parse type arguments for supertraits, so
                // we should give this error message to users.
                if !trait_decl.type_parameters.is_empty() {
                    errors.push(CompileError::Unimplemented(
                        "Using generic traits as supertraits is not supported yet.",
                        supertrait.name.span(),
                    ));
                    continue;
                }

                // TODO: right now supertraits can't take type arguments
                let mut type_arguments = vec![];

                // Monomorphize the trait declaration.
                check!(
                    ctx.monomorphize(
                        &mut trait_decl,
                        &mut type_arguments,
                        EnforceTypeArguments::Yes,
                        &supertrait.name.span()
                    ),
                    continue,
                    warnings,
                    errors
                );

                // Insert the interface surface and methods from this trait into
                // the namespace.
                trait_decl.insert_interface_surface_and_items_into_namespace(
                    ctx.by_ref(),
                    &supertrait.name,
                    &type_arguments,
                    type_id,
                );

                // Recurse to insert versions of interfaces and methods of the
                // *super* supertraits.
                check!(
                    insert_supertraits_into_namespace(
                        ctx.by_ref(),
                        type_id,
                        &trait_decl.supertraits,
                        &SupertraitOf::Trait
                    ),
                    continue,
                    warnings,
                    errors
                );
            }
            // an ABI can only be a superABI of an ABI
            (
                Some(ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. })),
                SupertraitOf::Abi(subabi_span),
            ) => {
                let abi_decl = decl_engine.get_abi(&decl_id);
                // Insert the interface surface and methods from this ABI into
                // the namespace.
                check!(
                    abi_decl.insert_interface_surface_and_items_into_namespace(
                        decl_id,
                        ctx.by_ref(),
                        type_id,
                        Some(subabi_span.clone())
                    ),
                    continue,
                    warnings,
                    errors
                );
                // Recurse to insert versions of interfaces and methods of the
                // *super* superABIs.
                check!(
                    insert_supertraits_into_namespace(
                        ctx.by_ref(),
                        type_id,
                        &abi_decl.supertraits,
                        &SupertraitOf::Abi(subabi_span.clone())
                    ),
                    continue,
                    warnings,
                    errors
                );
            }
            // an ABI cannot be a supertrait of a trait
            (Some(ty::TyDecl::AbiDecl { .. }), SupertraitOf::Trait) => {
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
