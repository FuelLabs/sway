use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;

use crate::{
    engine_threading::*,
    language::{parsed::Supertrait, ty, CallPath},
    semantic_analysis::{
        declaration::{insert_supertraits_into_namespace, SupertraitOf},
        type_check_context::EnforceTypeArguments,
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

impl DisplayWithEngines for TraitConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(self))
    }
}

impl DebugWithEngines for TraitConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let mut res = write!(f, "{}", self.trait_name);
        if !self.type_arguments.is_empty() {
            write!(f, "<")?;
            for ty_arg in self.type_arguments.clone() {
                write!(f, "{:?}", engines.help_out(ty_arg))?;
            }
            res = write!(f, ">");
        }
        res
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
        handler.scope(|handler| {
            for type_arg in self.type_arguments.iter() {
                res.extend(
                    match type_arg.type_id.collect_types_metadata(handler, ctx) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                );
            }
            Ok(res)
        })
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

        // Type check the type arguments.
        for type_argument in self.type_arguments.iter_mut() {
            type_argument.type_id = ctx
                .resolve_type(
                    handler,
                    type_argument.type_id,
                    &type_argument.span,
                    EnforceTypeArguments::Yes,
                    None,
                )
                .unwrap_or_else(|err| {
                    ctx.engines
                        .te()
                        .insert(ctx.engines(), TypeInfo::ErrorRecovery(err), None)
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
        let engines = ctx.engines;
        let decl_engine = engines.de();

        let TraitConstraint {
            trait_name,
            type_arguments,
        } = trait_constraint;

        let mut type_arguments = type_arguments.clone();

        match ctx
            .namespace
            .resolve_call_path(handler, engines, trait_name, ctx.self_type())
            .ok()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let mut trait_decl = decl_engine.get_trait(&decl_id);

                // the following essentially is needed to map `Self` to the right type
                // during trait decl monomorphization
                trait_decl
                    .type_parameters
                    .push(trait_decl.self_type.clone());
                type_arguments.push(TypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: trait_name.span(),
                    call_path_tree: None,
                });

                // Monomorphize the trait declaration.
                ctx.monomorphize(
                    handler,
                    &mut trait_decl,
                    &mut type_arguments,
                    EnforceTypeArguments::Yes,
                    &trait_name.span(),
                )?;

                // restore type parameters and type arguments
                trait_decl.type_parameters.pop();
                type_arguments.pop();

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
