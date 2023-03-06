use std::fmt;

use crate::language::ty::{TyTraitInterfaceItem, TyTraitItem};

use super::{FunctionalDeclId, InterfaceItemMap, ItemMap};

type SourceDecl = FunctionalDeclId;
type DestinationDecl = FunctionalDeclId;

/// The [DeclMapping] is used to create a mapping between a [SourceDecl] (LHS)
/// and a [DestinationDecl] (RHS).
pub struct DeclMapping {
    mapping: Vec<(SourceDecl, DestinationDecl)>,
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
                        source_type,
                        match dest_type {
                            FunctionalDeclId::TraitFn(decl_id) => decl_id.inner(),
                            FunctionalDeclId::Function(decl_id) => decl_id.inner(),
                            FunctionalDeclId::Constant(decl_id) => decl_id.inner(),
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

impl DeclMapping {
    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub(crate) fn from_interface_and_item_and_impld_decl_refs(
        interface_decl_refs: InterfaceItemMap,
        item_decl_refs: ItemMap,
        impld_decl_refs: ItemMap,
    ) -> DeclMapping {
        let mut mapping = vec![];
        for (interface_decl_name, interface_item) in interface_decl_refs.into_iter() {
            if let Some(new_item) = impld_decl_refs.get(&interface_decl_name) {
                let interface_decl_ref = match interface_item {
                    TyTraitInterfaceItem::TraitFn(decl_ref) => decl_ref.id.into(),
                    TyTraitInterfaceItem::Constant(decl_ref) => decl_ref.id.into(),
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id.into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id.into(),
                };
                mapping.push((interface_decl_ref, new_decl_ref));
            }
        }
        for (decl_name, item) in item_decl_refs.into_iter() {
            if let Some(new_item) = impld_decl_refs.get(&decl_name) {
                let interface_decl_ref = match item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id.into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id.into(),
                };
                let new_decl_ref = match new_item {
                    TyTraitItem::Fn(decl_ref) => decl_ref.id.into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id.into(),
                };
                mapping.push((interface_decl_ref, new_decl_ref));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn find_match(&self, decl_ref: SourceDecl) -> Option<DestinationDecl> {
        for (source_decl_ref, dest_decl_ref) in self.mapping.iter() {
            if *source_decl_ref == decl_ref {
                return Some(dest_decl_ref.clone());
            }
        }
        None
    }
}
