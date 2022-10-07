use sway_types::Ident;

use crate::{language::parsed::DepName, namespace};

use super::ast_node::TyAstNode;

#[derive(Clone, Debug)]
pub struct TyModule {
    pub submodules: Vec<(DepName, TySubmodule)>,
    pub namespace: namespace::Module,
    pub all_nodes: Vec<TyAstNode>,
}

#[derive(Clone, Debug)]
pub struct TySubmodule {
    pub library_name: Ident,
    pub module: TyModule,
}
