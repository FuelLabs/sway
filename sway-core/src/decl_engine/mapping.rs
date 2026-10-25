use std::{collections::HashSet, fmt};

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    engine_threading::DebugWithEngines,
    language::ty::{TyTraitInterfaceItem, TyTraitItem},
    Engines, TypeId, UnifyCheck,
};

use super::{FunctionalDeclId, InterfaceItemMap, ItemMap};

type SourceDecl = (FunctionalDeclId, TypeId);
type DestinationDecl = FunctionalDeclId;

/// The [DeclMapping] is used to create a mapping between a [SourceDecl] (LHS)
/// and a [DestinationDecl] (RHS).
///
/// Note that [DeclMapping] is **not a mapping of arbitrary `DeclId`s**.
/// Its whole domain is [FunctionalDeclId], which is only the function-like
/// trait/impl associated items: trait interface functions and functions.
/// [DeclMapping] maps a trait interface function reference to its concrete
/// impl counterpart.
///
/// For concrete swaps of trait interface function references with their concrete
/// impls, see [find_match], which is the final step of the trait-method
/// monomorphization.
///
/// Constants and associated types, although also being trait/impl associated
/// items, are deliberately not mapped. They are resolved by different means:
/// constants by name and associated types via the type substitution within
/// the [crate::TypeEngine].
#[derive(Clone)]
pub struct DeclMapping {
    pub mapping: Vec<(SourceDecl, DestinationDecl)>,
}

impl fmt::Display for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeclMapping {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| {
                    format!(
                        "{} -> {}",
                        source_type.0,
                        match dest_type {
                            FunctionalDeclId::TraitFn(decl_id) => decl_id.inner(),
                            FunctionalDeclId::Function(decl_id) => decl_id.inner(),
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl fmt::Debug for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeclMapping {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| { format!("{source_type:?} -> {dest_type:?}") })
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl DebugWithEngines for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        f.write_str("DeclMapping ").unwrap();
        let mut map = f.debug_map();
        for (source_type, dest_type) in self.mapping.iter() {
            let key = engines.help_out(source_type.0.clone());
            let value = engines.help_out(dest_type);
            map.entry(&key, &value);
        }
        map.finish()
    }
}

impl DeclMapping {
    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub(crate) fn extend(&mut self, other: &DeclMapping) {
        self.mapping.extend(other.mapping.clone());
    }

    pub(crate) fn from_interface_and_item_and_impld_decl_refs(
        interface_decl_refs: InterfaceItemMap,
        item_decl_refs: ItemMap,
        impld_decl_refs: ItemMap,
    ) -> DeclMapping {
        let mut mapping: Vec<(SourceDecl, DestinationDecl)> = vec![];
        for (interface_decl_name, interface_item) in interface_decl_refs.into_iter() {
            if let Some(new_item) = impld_decl_refs.get(&interface_decl_name) {
                // Only functions are mapped. Constants and associated types are
                // resolved by different means. See the [DeclMapping] documentation.
                let interface_decl_ref = match interface_item {
                    TyTraitInterfaceItem::TraitFn(decl_ref) => {
                        (decl_ref.id().into(), interface_decl_name.1)
                    }
                    TyTraitInterfaceItem::Constant(_) | TyTraitInterfaceItem::Type(_) => continue,
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Constant(_) | TyTraitItem::Type(_) => continue,
                };
                mapping.push((interface_decl_ref, new_decl_ref));
            }
        }
        for (decl_name, item) in item_decl_refs.into_iter() {
            if let Some(new_item) = impld_decl_refs.get(&decl_name) {
                // Only functions are mapped. Constants and associated types are
                // resolved by different means. See the [DeclMapping] documentation.
                let interface_decl_ref = match item {
                    TyTraitItem::Fn(decl_ref) => (decl_ref.id().into(), decl_name.1),
                    TyTraitItem::Constant(_) | TyTraitItem::Type(_) => continue,
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Constant(_) | TyTraitItem::Type(_) => continue,
                };
                mapping.push((interface_decl_ref, new_decl_ref));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn find_match(
        &self,
        _handler: &Handler,
        engines: &Engines,
        decl_ref: FunctionalDeclId,
        typeid: Option<TypeId>,
        self_typeid: Option<TypeId>,
    ) -> Result<Option<DestinationDecl>, ErrorEmitted> {
        let mut dest_decl_refs = HashSet::<DestinationDecl>::new();

        if let Some(mut typeid) = typeid {
            if let Some(self_ty) = self_typeid {
                if engines.te().get(typeid).is_self_type() {
                    // If typeid is `Self`, then we use the self_typeid instead.
                    typeid = self_ty;
                }
            }
            for (source_decl_ref, dest_decl_ref) in self.mapping.iter() {
                let unify_check = UnifyCheck::non_dynamic_equality(engines);
                if source_decl_ref.0 == decl_ref && unify_check.check(source_decl_ref.1, typeid) {
                    dest_decl_refs.insert(dest_decl_ref.clone());
                }
            }
        }

        // At most one replacement should be found for decl_ref.
        /* TODO uncomment this and close issue #5540
        if dest_decl_refs.len() > 1 {
            handler.emit_err(CompileError::InternalOwned(
                format!(
                    "Multiple replacements for decl {} implemented in {}",
                    engines.help_out(decl_ref),
                    engines.help_out(typeid),
                ),
                dest_decl_refs.iter().last().unwrap().span(engines),
            ));
        }*/
        Ok(dest_decl_refs.iter().next().cloned())
    }
}
