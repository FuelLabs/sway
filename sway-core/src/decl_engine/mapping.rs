use std::{collections::HashSet, fmt};

use sway_error::handler::{ErrorEmitted, Handler};

use crate::{
    engine_threading::DebugWithEngines,
    language::ty::{TyTraitInterfaceItem, TyTraitItem},
    Engines, TypeId, UnifyCheck,
};

use super::{AssociatedItemDeclId, InterfaceItemMap, ItemMap};

type SourceDecl = (AssociatedItemDeclId, TypeId);
type DestinationDecl = AssociatedItemDeclId;

/// The [DeclMapping] is used to create a mapping between a [SourceDecl] (LHS)
/// and a [DestinationDecl] (RHS).
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
                            AssociatedItemDeclId::TraitFn(decl_id) => decl_id.inner(),
                            AssociatedItemDeclId::Function(decl_id) => decl_id.inner(),
                            AssociatedItemDeclId::Constant(decl_id) => decl_id.inner(),
                            AssociatedItemDeclId::Type(decl_id) => decl_id.inner(),
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
                let interface_decl_ref = match interface_item {
                    TyTraitInterfaceItem::TraitFn(decl_ref) => {
                        (decl_ref.id().into(), interface_decl_name.1)
                    }
                    TyTraitInterfaceItem::Constant(decl_ref) => {
                        (decl_ref.id().into(), interface_decl_name.1)
                    }
                    TyTraitInterfaceItem::Type(decl_ref) => {
                        (decl_ref.id().into(), interface_decl_name.1)
                    }
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Type(decl_ref) => decl_ref.id().into(),
                };
                mapping.push((interface_decl_ref, new_decl_ref));
            }
        }
        for (decl_name, item) in item_decl_refs.into_iter() {
            if let Some(new_item) = impld_decl_refs.get(&decl_name) {
                let interface_decl_ref = match item {
                    TyTraitItem::Fn(decl_ref) => (decl_ref.id().into(), decl_name.1),
                    TyTraitItem::Constant(decl_ref) => (decl_ref.id().into(), decl_name.1),
                    TyTraitItem::Type(decl_ref) => (decl_ref.id().into(), decl_name.1),
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Type(decl_ref) => decl_ref.id().into(),
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
        decl_ref: AssociatedItemDeclId,
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
