use sway_error::error::CompileError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{Span, Spanned};

use crate::semantic_analysis::type_check_context::EnforceTypeArguments;
use crate::{
    language::{parsed, ty},
    semantic_analysis::TypeCheckContext,
    TypeId,
};

#[derive(Clone, PartialEq, Eq)]
pub enum SupertraitOf {
    Abi(Span), // Span is needed for error reporting
    Trait,
}

/// Recursively insert the interface surfaces and methods from supertraits to
/// the given namespace.
pub(crate) fn insert_supertraits_into_namespace(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    type_id: TypeId,
    supertraits: &[parsed::Supertrait],
    supertraits_of: &SupertraitOf,
) -> Result<(), ErrorEmitted> {
    let engines = ctx.engines;
    let decl_engine = engines.de();

    handler.scope(|handler| {
        for supertrait in supertraits.iter() {
            // Right now we don't have the ability to support defining a supertrait
            // using a callpath directly, so we check to see if the user has done
            // this and we disallow it.
            if !supertrait.name.prefixes.is_empty() {
                handler.emit_err(CompileError::Unimplemented {
                    feature: "Using module paths to define supertraits".to_string(),
                    help: vec![
                        // Note that eventual leading `::` will not be shown. It'a fine for now, we anyhow want to implement using module paths.
                        format!(
                            "Import the supertrait by using: `use {};`.",
                            supertrait.name
                        ),
                        format!(
                            "Then, in the list of supertraits, just use the trait name \"{}\".",
                            supertrait.name.suffix
                        ),
                    ],
                    span: supertrait.span(),
                });

                continue;
            }

            let decl = ctx
                .namespace()
                // Use the default Handler to avoid emitting the redundant SymbolNotFound error.
                .resolve_call_path_typed(
                    &Handler::default(),
                    engines,
                    &supertrait.name,
                    ctx.self_type(),
                )
                .ok();

            match (decl.clone(), supertraits_of) {
                // a trait can be a supertrait of either a trait or a an ABI
                (Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })), _) => {
                    let mut trait_decl = (*decl_engine.get_trait(&decl_id)).clone();

                    // Right now we don't parse type arguments for supertraits, so
                    // we should give this error message to users.
                    if !trait_decl.type_parameters.is_empty() {
                        handler.emit_err(CompileError::Unimplemented {
                            feature: "Using generic traits as supertraits".to_string(),
                            help: vec![],
                            span: supertrait.span(),
                        });

                        continue;
                    }

                    // TODO: right now supertraits can't take type arguments
                    let mut type_arguments = vec![];

                    // Monomorphize the trait declaration.
                    if ctx
                        .monomorphize(
                            handler,
                            &mut trait_decl,
                            &mut type_arguments,
                            EnforceTypeArguments::Yes,
                            &supertrait.name.span(),
                        )
                        .is_err()
                    {
                        continue;
                    }

                    // Insert the interface surface and methods from this trait into
                    // the namespace.
                    trait_decl.insert_interface_surface_and_items_into_namespace(
                        handler,
                        ctx.by_ref(),
                        &supertrait.name,
                        &type_arguments,
                        type_id,
                    );

                    // Recurse to insert versions of interfaces and methods of the
                    // *super* supertraits.
                    if insert_supertraits_into_namespace(
                        handler,
                        ctx.by_ref(),
                        type_id,
                        &trait_decl.supertraits,
                        &SupertraitOf::Trait,
                    )
                    .is_err()
                    {
                        continue;
                    }
                }
                // an ABI can only be a superABI of an ABI
                (
                    Some(ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. })),
                    SupertraitOf::Abi(subabi_span),
                ) => {
                    let abi_decl = decl_engine.get_abi(&decl_id);
                    // Insert the interface surface and methods from this ABI into
                    // the namespace.
                    if abi_decl
                        .insert_interface_surface_and_items_into_namespace(
                            handler,
                            decl_id,
                            ctx.by_ref(),
                            type_id,
                            Some(subabi_span.clone()),
                        )
                        .is_err()
                    {
                        continue;
                    }
                    // Recurse to insert versions of interfaces and methods of the
                    // *super* superABIs.

                    if insert_supertraits_into_namespace(
                        handler,
                        ctx.by_ref(),
                        type_id,
                        &abi_decl.supertraits,
                        &SupertraitOf::Abi(subabi_span.clone()),
                    )
                    .is_err()
                    {
                        continue;
                    }
                }
                // an ABI cannot be a supertrait of a trait
                (Some(ty::TyDecl::AbiDecl { .. }), SupertraitOf::Trait) => {
                    handler.emit_err(CompileError::AbiAsSupertrait {
                        span: supertrait.name.span().clone(),
                    });
                }
                _ => {
                    handler.emit_err(CompileError::TraitNotFound {
                        name: supertrait.name.to_string(),
                        span: supertrait.name.span(),
                    });
                }
            }
        }

        Ok(())
    })
}
