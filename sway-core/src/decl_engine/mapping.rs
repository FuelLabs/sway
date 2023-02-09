use std::fmt;

use super::{DeclId, DeclRef, MethodMap};

type SourceDecl = DeclRef;
type DestinationDecl = DeclRef;

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
                .map(|(source_type, dest_type)| {
                    format!(
                        "{} -> {}",
                        *DeclId::from(source_type),
                        *DeclId::from(dest_type)
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

    pub(crate) fn from_stub_and_impld_decl_refs(
        stub_decl_refs: MethodMap,
        impld_decl_refs: MethodMap,
    ) -> DeclMapping {
        let mut mapping = vec![];
        for (stub_decl_name, stub_decl_ref) in stub_decl_refs.into_iter() {
            if let Some(new_decl_ref) = impld_decl_refs.get(&stub_decl_name) {
                mapping.push((stub_decl_ref, new_decl_ref.clone()));
            }
        }
        DeclMapping { mapping }
    }

    pub(crate) fn find_match(&self, decl_ref: &SourceDecl) -> Option<DestinationDecl> {
        for (source_decl_ref, dest_decl_ref) in self.mapping.iter() {
            if *DeclId::from(source_decl_ref) == *DeclId::from(decl_ref) {
                return Some(dest_decl_ref.clone());
            }
        }
        None
    }
}
