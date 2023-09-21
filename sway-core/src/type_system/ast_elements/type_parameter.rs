use crate::{
    decl_engine::*,
    engine_threading::*,
    language::{ty, CallPath},
    namespace::TryInsertingTraitImplOnFailure,
    semantic_analysis::*,
    type_system::priv_prelude::*,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{ident::Ident, span::Span, Spanned};

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub struct TypeParameter {
    pub type_id: TypeId,
    pub(crate) initial_type_id: TypeId,
    pub name_ident: Ident,
    pub(crate) trait_constraints: Vec<TraitConstraint>,
    pub(crate) trait_constraints_span: Span,
    pub(crate) is_from_parent: bool,
}

impl HashWithEngines for TypeParameter {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let TypeParameter {
            type_id,
            name_ident,
            trait_constraints,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
        name_ident.hash(state);
        trait_constraints.hash(state, engines);
    }
}

impl EqWithEngines for TypeParameter {}
impl PartialEqWithEngines for TypeParameter {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        let type_engine = engines.te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), engines)
            && self.name_ident == other.name_ident
            && self.trait_constraints.eq(&other.trait_constraints, engines)
    }
}

impl OrdWithEngines for TypeParameter {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        let TypeParameter {
            type_id: lti,
            name_ident: ln,
            trait_constraints: ltc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = self;
        let TypeParameter {
            type_id: rti,
            name_ident: rn,
            trait_constraints: rtc,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            trait_constraints_span: _,
            initial_type_id: _,
            is_from_parent: _,
        } = other;
        ln.cmp(rn)
            .then_with(|| engines.te().get(*lti).cmp(&engines.te().get(*rti), engines))
            .then_with(|| ltc.cmp(rtc, engines))
    }
}

impl SubstTypes for TypeParameter {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: &Engines) {
        self.type_id.subst(type_mapping, engines);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.subst(type_mapping, engines));
    }
}

impl ReplaceSelfType for TypeParameter {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
        self.trait_constraints
            .iter_mut()
            .for_each(|x| x.replace_self_type(engines, self_type));
    }
}

impl Spanned for TypeParameter {
    fn span(&self) -> Span {
        self.name_ident.span()
    }
}

impl DebugWithEngines for TypeParameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        write!(
            f,
            "{}: {:?}",
            self.name_ident,
            engines.help_out(self.type_id)
        )
    }
}

impl fmt::Debug for TypeParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name_ident, self.type_id)
    }
}

impl TypeParameter {
    /// Type check a list of [TypeParameter] and return a new list of
    /// [TypeParameter]. This will also insert this new list into the current
    /// namespace.
    pub(crate) fn type_check_type_params(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_params: Vec<TypeParameter>,
    ) -> Result<Vec<TypeParameter>, ErrorEmitted> {
        let mut new_type_params: Vec<TypeParameter> = vec![];

        handler.scope(|handler| {
            for type_param in type_params.into_iter() {
                new_type_params.push(
                    match TypeParameter::type_check(handler, ctx.by_ref(), type_param) {
                        Ok(res) => res,
                        Err(_) => continue,
                    },
                )
            }

            Ok(new_type_params)
        })
    }

    /// Type checks a [TypeParameter] (including its [TraitConstraint]s) and
    /// inserts into into the current namespace.
    fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        type_parameter: TypeParameter,
    ) -> Result<Self, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let engines = ctx.engines();

        let TypeParameter {
            initial_type_id,
            name_ident,
            mut trait_constraints,
            trait_constraints_span,
            is_from_parent,
            ..
        } = type_parameter;

        // Type check the trait constraints.
        for trait_constraint in trait_constraints.iter_mut() {
            trait_constraint.type_check(handler, ctx.by_ref())?;
        }

        // TODO: add check here to see if the type parameter has a valid name and does not have type parameters

        let type_id = type_engine.insert(
            engines,
            TypeInfo::UnknownGeneric {
                name: name_ident.clone(),
                trait_constraints: VecSet(trait_constraints.clone()),
            },
        );

        // When type parameter is from parent then it was already inserted.
        // Instead of inserting a type with same name we unify them.
        if is_from_parent {
            if let Some(sy) = ctx.namespace.symbols.get(&name_ident) {
                match sy {
                    ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                        type_id: sy_type_id,
                        ..
                    }) => {
                        ctx.engines().te().unify(
                            handler,
                            ctx.engines(),
                            type_id,
                            *sy_type_id,
                            &trait_constraints_span,
                            "",
                            None,
                        );
                    }
                    _ => {
                        handler.emit_err(CompileError::Internal(
                            "Unexpected TyDeclaration for TypeParameter.",
                            name_ident.span(),
                        ));
                    }
                }
            }
        }

        let type_parameter = TypeParameter {
            name_ident,
            type_id,
            initial_type_id,
            trait_constraints,
            trait_constraints_span,
            is_from_parent,
        };
        Ok(type_parameter)
    }

    pub fn insert_into_namespace(
        &self,
        handler: &Handler,
        mut ctx: TypeCheckContext,
    ) -> Result<(), ErrorEmitted> {
        let Self {
            is_from_parent,
            name_ident,
            type_id,
            ..
        } = self;

        // Insert the trait constraints into the namespace.
        for trait_constraint in self.trait_constraints.iter() {
            TraitConstraint::insert_into_namespace(
                handler,
                ctx.by_ref(),
                *type_id,
                trait_constraint,
            )?;
        }

        if !is_from_parent {
            // Insert the type parameter into the namespace as a dummy type
            // declaration.
            let type_parameter_decl =
                ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                    name: name_ident.clone(),
                    type_id: *type_id,
                });
            ctx.insert_symbol(handler, name_ident.clone(), type_parameter_decl)
                .ok();
        }

        Ok(())
    }

    /// Creates a [DeclMapping] from a list of [TypeParameter]s.
    pub(crate) fn gather_decl_mapping_from_trait_constraints(
        handler: &Handler,
        ctx: TypeCheckContext,
        type_parameters: &[TypeParameter],
        access_span: &Span,
    ) -> Result<DeclMapping, ErrorEmitted> {
        let mut interface_item_refs: InterfaceItemMap = BTreeMap::new();
        let mut item_refs: ItemMap = BTreeMap::new();
        let mut impld_item_refs: ItemMap = BTreeMap::new();
        let engines = ctx.engines();

        handler.scope(|handler| {
            for type_param in type_parameters.iter() {
                let TypeParameter {
                    type_id,
                    trait_constraints,
                    ..
                } = type_param;

                // Check to see if the trait constraints are satisfied.
                match ctx
                    .namespace
                    .implemented_traits
                    .check_if_trait_constraints_are_satisfied_for_type(
                        handler,
                        *type_id,
                        trait_constraints,
                        access_span,
                        engines,
                        TryInsertingTraitImplOnFailure::Yes,
                    ) {
                    Ok(res) => res,
                    Err(_) => continue,
                }

                for trait_constraint in trait_constraints.iter() {
                    let TraitConstraint {
                        trait_name,
                        type_arguments: trait_type_arguments,
                    } = trait_constraint;

                    let (trait_interface_item_refs, trait_item_refs, trait_impld_item_refs) =
                        match handle_trait(
                            handler,
                            &ctx,
                            *type_id,
                            trait_name,
                            trait_type_arguments,
                        ) {
                            Ok(res) => res,
                            Err(_) => continue,
                        };
                    interface_item_refs.extend(trait_interface_item_refs);
                    item_refs.extend(trait_item_refs);
                    impld_item_refs.extend(trait_impld_item_refs);
                }
            }

            let decl_mapping = DeclMapping::from_interface_and_item_and_impld_decl_refs(
                interface_item_refs,
                item_refs,
                impld_item_refs,
            );
            Ok(decl_mapping)
        })
    }
}

fn handle_trait(
    handler: &Handler,
    ctx: &TypeCheckContext,
    type_id: TypeId,
    trait_name: &CallPath,
    type_arguments: &[TypeArgument],
) -> Result<(InterfaceItemMap, ItemMap, ItemMap), ErrorEmitted> {
    let engines = ctx.engines;
    let decl_engine = engines.de();

    let mut interface_item_refs: InterfaceItemMap = BTreeMap::new();
    let mut item_refs: ItemMap = BTreeMap::new();
    let mut impld_item_refs: ItemMap = BTreeMap::new();

    handler.scope(|handler| {
        match ctx
            .namespace
            .resolve_call_path(handler, engines, trait_name)
            .ok()
        {
            Some(ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. })) => {
                let trait_decl = decl_engine.get_trait(&decl_id);

                let (trait_interface_item_refs, trait_item_refs, trait_impld_item_refs) =
                    trait_decl.retrieve_interface_surface_and_items_and_implemented_items_for_type(
                        ctx,
                        type_id,
                        trait_name,
                        type_arguments,
                    );
                interface_item_refs.extend(trait_interface_item_refs);
                item_refs.extend(trait_item_refs);
                impld_item_refs.extend(trait_impld_item_refs);

                for supertrait in trait_decl.supertraits.iter() {
                    let (
                        supertrait_interface_item_refs,
                        supertrait_item_refs,
                        supertrait_impld_item_refs,
                    ) = match handle_trait(handler, ctx, type_id, &supertrait.name, &[]) {
                        Ok(res) => res,
                        Err(_) => continue,
                    };
                    interface_item_refs.extend(supertrait_interface_item_refs);
                    item_refs.extend(supertrait_item_refs);
                    impld_item_refs.extend(supertrait_impld_item_refs);
                }
            }
            _ => {
                handler.emit_err(CompileError::TraitNotFound {
                    name: trait_name.to_string(),
                    span: trait_name.span(),
                });
            }
        }

        Ok((interface_item_refs, item_refs, impld_item_refs))
    })
}
