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
