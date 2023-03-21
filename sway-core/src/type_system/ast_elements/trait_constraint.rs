use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    error::*,
    language::{parsed::Supertrait, ty, CallPath},
    semantic_analysis::{declaration::insert_supertraits_into_namespace, TypeCheckContext},
    type_system::priv_prelude::*,
    types::*,
    CompileResult,
};

#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub trait_name: CallPath,
    pub type_arguments: Vec<TypeArgument>,
}

impl HashWithEngines for TraitConstraint {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        self.trait_name.hash(state);
        self.type_arguments.hash(state, engines);
    }
}

impl EqWithEngines for TraitConstraint {}
impl PartialEqWithEngines for TraitConstraint {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.trait_name == other.trait_name
            && self.type_arguments.eq(&other.type_arguments, engines)
    }
}

impl OrdWithEngines for TraitConstraint {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TraitConstraint {
            trait_name: ltn,
            type_arguments: lta,
        } = self;
        let TraitConstraint {
            trait_name: rtn,
            type_arguments: rta,
        } = other;
        ltn.cmp(rtn).then_with(|| lta.cmp(rta, engines))
    }
}

impl Spanned for TraitConstraint {
    fn span(&self) -> sway_types::Span {
        self.trait_name.span()
    }
}

impl From<&Supertrait> for TraitConstraint {
    fn from(supertrait: &Supertrait) -> Self {
        TraitConstraint {
            trait_name: supertrait.name.clone(),
            type_arguments: vec![],
        }
    }
}

impl CollectTypesMetadata for TraitConstraint {
    fn collect_types_metadata(
        &self,
        ctx: &mut CollectTypesMetadataContext,
    ) -> CompileResult<Vec<TypeMetadata>> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let mut res = vec![];
        for type_arg in self.type_arguments.iter() {
            res.extend(check!(
                type_arg.type_id.collect_types_metadata(ctx),
                continue,
                warnings,
                errors
            ));
        }
        if errors.is_empty() {
            ok(res, warnings, errors)
        } else {
            err(warnings, errors)
        }
    }
}

impl TraitConstraint {
    pub(crate) fn type_check(&mut self, mut ctx: TypeCheckContext) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let decl_engine = ctx.decl_engine;

        // Right now we don't have the ability to support defining a type for a
        // trait constraint using a callpath directly, so we check to see if the
        // user has done this and we disallow it.
        if !self.trait_name.prefixes.is_empty() {
            errors.push(CompileError::UnimplementedWithHelp(
                "Using module paths to define trait constraints is not supported yet.",
                "try importing the trait with a \"use\" statement instead",
                self.trait_name.span(),
            ));
            return err(warnings, errors);
        }

        // Right now we aren't supporting generic traits in trait constraints
        // because of how we type check trait constraints.
        // Essentially type checking trait constraints with generic traits
        // creates a chicken and an egg problem where, in order to type check
        // the type arguments to the generic traits, we must first type check
        // all of the type parameters, but we cannot finish type checking one
        // type parameter until we type check the trait constraints for that
        // type parameter. This is not an unsolvable problem, it will just
        // require some hacking.
        //
        // TODO: implement a fix for the above in a future PR
        if !self.type_arguments.is_empty() {
            errors.push(CompileError::Unimplemented(
                "Using generic traits in trait constraints is not supported yet.",
                Span::join_all(
                    self.type_arguments
                        .iter()
                        .map(|x| x.span())
                        .collect::<Vec<_>>(),
                ),
            ));
            return err(warnings, errors);
        }

        // Type check the type arguments.
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.type_id = check!(
                ctx.resolve_type(
                    type_argument.type_id,
                    &type_argument.span,
                    EnforceTypeArguments::Yes,
                    None
                ),
                ctx.type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                warnings,
                errors
            );
        }

        if errors.is_empty() {
            ok((), warnings, errors)
        } else {
            err(warnings, errors)
        }
    }

    pub(crate) fn insert_into_namespace(
        mut ctx: TypeCheckContext,
        type_id: TypeId,
        trait_constraint: &TraitConstraint,
    ) -> CompileResult<()> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        let TraitConstraint {
            trait_name,
            type_arguments,
        } = trait_constraint;

        let mut type_arguments = type_arguments.clone();

        match ctx
            .namespace
            .resolve_call_path(trait_name)
            .ok(&mut warnings, &mut errors)
            .cloned()
        {
            Some(ty::TyDecl::TraitDecl {
                name,
                decl_id,
                subst_list,
                decl_span,
            }) => {
                let mut trait_ref =
                    DeclRef::new(name, decl_id, subst_list.scoped_copy(engines), decl_span);
                // Monomorphize the trait declaration.
                check!(
                    ctx.combine_subst_list_and_args(
                        &mut trait_ref,
                        &mut type_arguments,
                        EnforceTypeArguments::Yes,
                        &trait_name.span()
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let trait_decl = decl_engine.get_trait(&trait_ref);
                // Insert the interface surface and methods from this trait into
                // the namespace.
                trait_decl.insert_interface_surface_and_items_into_namespace(
                    ctx.by_ref(),
                    trait_name,
                    &type_arguments,
                    type_id,
                );
                // Recursively make the interface surfaces and methods of the
                // supertraits available to this trait.
                check!(
                    insert_supertraits_into_namespace(
                        ctx.by_ref(),
                        type_id,
                        &trait_decl.supertraits
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
            Some(ty::TyDecl::AbiDecl { .. }) => errors.push(CompileError::AbiAsSupertrait {
                span: trait_name.span(),
            }),
            _ => errors.push(CompileError::TraitNotFound {
                name: trait_name.to_string(),
                span: trait_name.span(),
            }),
        }

        ok((), warnings, errors)
    }
}
