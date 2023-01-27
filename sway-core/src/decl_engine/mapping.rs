use std::{collections::BTreeMap, fmt};

use sway_types::Ident;

use crate::language::ty::TyFunctionSig;

use super::DeclId;

type SourceDecl = DeclId;
type DestinationDecl = DeclId;

/// The [DeclMapping] is used to create a mapping between a [SourceDecl] (LHS)
/// and a [DestinationDecl] (RHS).
pub(crate) struct DeclMapping {
    mapping: Vec<(SourceDecl, DestinationDecl)>,
}

impl fmt::Display for DeclMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeclMapping {{ {} }}",
            self.mapping
                .iter()
                .map(|(source_type, dest_type)| { format!("{} -> {}", **source_type, **dest_type) })
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

    pub(crate) fn mapping(&self) -> Vec<(SourceDecl, DestinationDecl)> {
        self.mapping.clone()
    }

    pub(crate) fn from_mapping(mapping: Vec<(SourceDecl, DestinationDecl)>) -> DeclMapping {
        DeclMapping { mapping }
    }

    pub(crate) fn from_stub_and_impld_decl_ids(
        stub_decl_ids: BTreeMap<Ident, DeclId>,
        impld_decl_ids: BTreeMap<Ident, DeclId>,
    ) -> DeclMapping {
        let mut mapping = vec![];
        for (stub_decl_name, stub_decl_id) in stub_decl_ids.into_iter() {
            if let Some(new_decl_id) = impld_decl_ids.get(&stub_decl_name) {
                mapping.push((stub_decl_id, new_decl_id.clone()));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn from_stub_and_impld_decl_ids_with_function_sig(
        stub_decl_ids: BTreeMap<Ident, DeclId>,
        new_decl_ids: BTreeMap<(Ident, TyFunctionSig), DeclId>,
    ) -> DeclMapping {
        let mut mapping = vec![];
        for (stub_decl_name, stub_decl_id) in stub_decl_ids.into_iter() {
            for ((new_decl_name, _), new_decl_id) in new_decl_ids.iter() {
                if new_decl_name.as_str() != stub_decl_name.as_str() {
                    continue;
                }
                mapping.push((stub_decl_id.clone(), new_decl_id.clone()));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn find_match(&self, decl_id: &SourceDecl) -> Option<DestinationDecl> {
        for (source_decl_id, dest_decl_id) in self.mapping.iter() {
            if **source_decl_id == **decl_id {
                return Some(dest_decl_id.clone());
            }
        }
        None
    }
}
