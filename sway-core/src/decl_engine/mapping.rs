use std::fmt;

use crate::{
    language::ty::{TyTraitInterfaceItem, TyTraitItem},
    Engines, TypeId, UnifyCheck,
};

use super::{AssociatedItemDeclId, DeclEngineGet, InterfaceItemMap, ItemMap};

type SourceDecl = AssociatedItemDeclId;
type DestinationDecl = AssociatedItemDeclId;

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

impl DeclMapping {
    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
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
                    TyTraitInterfaceItem::TraitFn(decl_ref) => decl_ref.id().into(),
                    TyTraitInterfaceItem::Constant(decl_ref) => decl_ref.id().into(),
                    TyTraitInterfaceItem::Type(decl_ref) => decl_ref.id().into(),
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
                    TyTraitItem::Fn(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Constant(decl_ref) => decl_ref.id().into(),
                    TyTraitItem::Type(decl_ref) => decl_ref.id().into(),
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

    pub(crate) fn find_match(&self, decl_ref: SourceDecl) -> Option<DestinationDecl> {
        for (source_decl_ref, dest_decl_ref) in self.mapping.iter() {
            if *source_decl_ref == decl_ref {
                return Some(dest_decl_ref.clone());
            }
        }
        None
    }

    /// This method returns only associated item functions that have as self type the given type.
    pub(crate) fn filter_functions_by_self_type(
        &self,
        self_type: TypeId,
        engines: &Engines,
    ) -> DeclMapping {
        let mut mapping: Vec<(SourceDecl, DestinationDecl)> = vec![];
        for (source_decl_ref, dest_decl_ref) in self.mapping.iter().cloned() {
            match dest_decl_ref {
                AssociatedItemDeclId::TraitFn(_) => mapping.push((source_decl_ref, dest_decl_ref)),
                AssociatedItemDeclId::Function(func_id) => {
                    let func = engines.de().get(&func_id);

                    let unify_check = UnifyCheck::non_dynamic_equality(engines);
                    if let (left, Some(right)) = (self_type, func.parameters.get(0)) {
                        if unify_check.check(left, right.type_argument.type_id) {
                            mapping.push((source_decl_ref, dest_decl_ref));
                        }
                    }
                }
                AssociatedItemDeclId::Constant(_) => mapping.push((source_decl_ref, dest_decl_ref)),
                AssociatedItemDeclId::Type(_) => mapping.push((source_decl_ref, dest_decl_ref)),
            }
        }
        DeclMapping { mapping }
    }
}
