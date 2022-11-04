use sway_error::error::CompileError;
use sway_types::Spanned;

use crate::{
    declaration_engine::*,
    error::*,
    language::{parsed, ty},
    semantic_analysis::TypeCheckContext,
    EnforceTypeArguments,
};

/// Recursively insert the interface surfaces and methods from supertraits to
/// the given namespace.
pub(crate) fn insert_supertraits_into_namespace(
    mut ctx: TypeCheckContext,
    supertraits: &[parsed::Supertrait],
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

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
                let mut trait_decl = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &supertrait.span())),
                    break,
                    warnings,
                    errors
                );

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
                check!(
                    trait_decl.insert_interface_surface_and_methods_into_namespace(
                        ctx.by_ref(),
                        &supertrait.name,
                        &type_arguments,
                        self_type
                    ),
                    continue,
                    warnings,
                    errors
                );

                // Recurse to insert versions of interfaces and methods of the
                // *super* supertraits.
                check!(
                    insert_supertraits_into_namespace(ctx.by_ref(), &trait_decl.supertraits),
                    continue,
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
