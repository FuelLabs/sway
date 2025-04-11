use crate::{
    engine_threading::*,
    language::{parsed::Supertrait, ty, CallPath, CallPathDisplayType},
    semantic_analysis::{
        declaration::{insert_supertraits_into_namespace, SupertraitOf},
        TypeCheckContext,
    },
    type_system::priv_prelude::*,
    types::{CollectTypesMetadata, CollectTypesMetadataContext, TypeMetadata},
    EnforceTypeArguments, Namespace,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Spanned;

use super::type_argument::GenericTypeArgument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitConstraint {
    pub trait_name: CallPath,
    pub type_arguments: Vec<GenericArgument>,
}

impl HashWithEngines for TraitConstraint {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        self.trait_name.hash(state);
        self.type_arguments.hash(state, engines);
    }
}

impl EqWithEngines for TraitConstraint {}
impl PartialEqWithEngines for TraitConstraint {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.trait_name == other.trait_name
            // Check if eq is already inside of a trait constraint, if it is we don't compare type arguments.
            // This breaks the recursion when we use a where clause such as `T:MyTrait<T>`.
            && (ctx.is_inside_trait_constraint()
                || self.type_arguments.eq(
                    &other.type_arguments,
                    &(ctx.with_is_inside_trait_constraint()),
                ))
    }
}

impl OrdWithEngines for TraitConstraint {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        let TraitConstraint {
            trait_name: ltn,
            type_arguments: lta,
        } = self;
        let TraitConstraint {
            trait_name: rtn,
            type_arguments: rta,
        } = other;
        let mut res = ltn.cmp(rtn);

        // Check if cmp is already inside of a trait constraint, if it is we don't compare type arguments.
        // This breaks the recursion when we use a where clause such as `T:MyTrait<T>`.
        if !ctx.is_inside_trait_constraint() {
            res = res.then_with(|| lta.cmp(rta, &ctx.with_is_inside_trait_constraint()));
        }

        res
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
    fn subst_inner(&mut self, ctx: &SubstTypesContext) -> HasChanges {
        self.type_arguments.subst(ctx)
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
            for type_arg in &self.type_arguments {
                res.extend(
                    match type_arg.type_id().collect_types_metadata(handler, ctx) {
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
        ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        // Right now we don't have the ability to support defining a type for a
        // trait constraint using a callpath directly, so we check to see if the
        // user has done this and we disallow it.
        if !self.trait_name.prefixes.is_empty() {
            return Err(handler.emit_err(CompileError::Unimplemented {
                feature: "Using module paths to define trait constraints".to_string(),
                help: vec![
                    // Note that eventual leading `::` will not be shown. It'a fine for now, we anyhow want to implement using module paths.
                    format!(
                        "Import the supertrait by using: `use {};`.",
                        self.trait_name
                    ),
                    format!(
                        "Then, in the trait constraints, just use the trait name \"{}\".",
                        self.trait_name.suffix
                    ),
                ],
                span: self.trait_name.span(),
            }));
        }

        self.trait_name = self
            .trait_name
            .to_canonical_path(ctx.engines(), ctx.namespace());

        // Type check the type arguments.
        for type_argument in &mut self.type_arguments {
            *type_argument.type_id_mut() = ctx
                .resolve_type(
                    handler,
                    type_argument.type_id(),
                    &type_argument.span(),
                    EnforceTypeArguments::Yes,
                    None,
                )
                .unwrap_or_else(|err| ctx.engines.te().id_of_error_recovery(err));
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
            // Use the default Handler to avoid emitting the redundant SymbolNotFound error.
            .resolve_call_path(&Handler::default(), trait_name)
            .ok()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let mut trait_decl = (*decl_engine.get_trait(&decl_id)).clone();

                // the following essentially is needed to map `Self` to the right type
                // during trait decl monomorphization
                trait_decl
                    .type_parameters
                    .push(trait_decl.self_type.clone());
                type_arguments.push(GenericArgument::Type(GenericTypeArgument {
                    type_id,
                    initial_type_id: type_id,
                    span: trait_name.span(),
                    call_path_tree: None,
                }));

                // Monomorphize the trait declaration.
                ctx.monomorphize(
                    handler,
                    &mut trait_decl,
                    &mut type_arguments,
                    BTreeMap::new(),
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

    pub fn to_display_name(&self, engines: &Engines, namespace: &Namespace) -> String {
        let display_path = self
            .trait_name
            .to_display_path(CallPathDisplayType::StripPackagePrefix, namespace);
        display_path.to_string_with_args(engines, &self.type_arguments)
    }
}
