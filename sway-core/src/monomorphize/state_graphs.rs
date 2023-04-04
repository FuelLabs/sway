use petgraph::Graph;

use crate::{decl_engine::DeclId, language::ty::*, SubstList};

pub(crate) struct StateGraphs {
    /// Graph representing the flow of monomorphization for functions.
    pub(super) fn_graph: Graph<DeclId<TyFunctionDecl>, Option<SubstList>>,

    /// Graph representing the flow of monomorphization for structs.
    pub(super) struct_graph: Graph<DeclId<TyStructDecl>, Option<SubstList>>,

    /// Graph representing the flow of monomorphization for enums.
    pub(super) enum_graph: Graph<DeclId<TyEnumDecl>, Option<SubstList>>,

    /// Graph representing the flow of monomorphization for traits.
    pub(super) trait_graph: Graph<DeclId<TyTraitDecl>, Option<SubstList>>,
}
