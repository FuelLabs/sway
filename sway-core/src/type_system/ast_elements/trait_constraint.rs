use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Span, Spanned};

use crate::{
    engine_threading::*,
    language::{parsed::Supertrait, ty, CallPath},
    semantic_analysis::{
        declaration::{insert_supertraits_into_namespace, SupertraitOf},
        TypeCheckContext,
    },
    type_system::priv_prelude::*,
    types::*,
};

#[derive(Debug, Clone)]
pub struct TraitConstraint {
    pub trait_name: CallPath,
    pub type_arguments: Vec<TypeArgument>,
}

impl HashWithEngines for TraitConstraint {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.trait_name.hash(state);
        self.type_arguments.hash(state, engines);
    }
}

impl EqWithEngines for TraitConstraint {}
impl PartialEqWithEngines for TraitConstraint {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.trait_name == other.trait_name
            && self.type_arguments.eq(&other.type_arguments, engines)
    }
}

impl OrdWithEngines for TraitConstraint {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
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

impl SubstTypes for TraitConstraint {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.type_arguments
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TraitConstraint {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        self.type_arguments
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
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
        handler: &Handler,
        ctx: &mut CollectTypesMetadataContext,
    ) -> Result<Vec<TypeMetadata>, ErrorEmitted> {
        let mut res = vec![];
        let mut error_emitted = None;
        for type_arg in self.type_arguments.iter() {
            res.extend(
                match type_arg.type_id.collect_types_metadata(handler, ctx) {
                    Ok(res) => res,
                    Err(err) => {
                        error_emitted = Some(err);
                        continue;
                    }
                },
            );
        }

        if let Some(err) = error_emitted {
            Err(err)
        } else {
            Ok(res)
        }
    }
}

impl TraitConstraint {
    pub(crate) fn type_check(
        &mut self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        // Right now we don't have the ability to support defining a type for a
        // trait constraint using a callpath directly, so we check to see if the
        // user has done this and we disallow it.
        if !self.trait_name.prefixes.is_empty() {
            return Err(handler.emit_err(CompileError::UnimplementedWithHelp(
                "Using module paths to define trait constraints is not supported yet.",
                "try importing the trait with a \"use\" statement instead",
                self.trait_name.span(),
            )));
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
            return Err(handler.emit_err(CompileError::Unimplemented(
                "Using generic traits in trait constraints is not supported yet.",
                Span::join_all(
                    self.type_arguments
                        .iter()
                        .map(|x| x.span())
                        .collect::<Vec<_>>(),
                ),
            )));
        }

        // Type check the type arguments.
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.type_id = ctx
                .resolve_type_without_self(
                    handler,
                    type_argument.type_id,
                    &type_argument.span,
                    None,
                )
                .unwrap_or_else(|_| {
                    ctx.engines
                        .te()
                        .insert(ctx.engines(), TypeInfo::ErrorRecovery)
                });
        }

        Ok(())
    }

    pub(crate) fn insert_into_namespace(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_id: TypeId,
        trait_constraint: &TraitConstraint,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();

        let TraitConstraint {
            trait_name,
            type_arguments,
        } = trait_constraint;

        let mut type_arguments = type_arguments.clone();

        match ctx
            .namespace
            .resolve_call_path(handler, trait_name)
            .ok()
            .cloned()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let mut trait_decl = decl_engine.get_trait(&decl_id);

                // Monomorphize the trait declaration.
                ctx.monomorphize(
                    handler,
                    &mut trait_decl,
                    &mut type_arguments,
                    EnforceTypeArguments::Yes,
                    &trait_name.span(),
                )?;

                // Insert the interface surface and methods from this trait into
                // the namespace.
                trait_decl.insert_interface_surface_and_items_into_namespace(
                    handler,
                    ctx.by_ref(),
                    trait_name,
                    &type_arguments,
                    type_id,
                );

                // Recursively make the interface surfaces and methods of the
                // supertraits available to this trait.
                insert_supertraits_into_namespace(
                    handler,
                    ctx.by_ref(),
                    type_id,
                    &trait_decl.supertraits,
                    &SupertraitOf::Trait,
                )?;
            }
            Some(ty::TyDecl::AbiDecl { .. }) => {
                handler.emit_err(CompileError::AbiAsSupertrait {
                    span: trait_name.span(),
                });
            }
            _ => {
                handler.emit_err(CompileError::TraitNotFound {
                    name: trait_name.to_string(),
                    span: trait_name.span(),
                });
            }
        }

        Ok(())
    }
}
