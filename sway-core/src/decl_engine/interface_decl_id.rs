use crate::{decl_engine::*, language::ty};

#[derive(Debug, Eq, PartialEq, Hash, Clone, deepsize::DeepSizeOf)]
pub enum InterfaceDeclId {
    Abi(DeclId<ty::TyAbiDecl>),
    Trait(DeclId<ty::TyTraitDecl>),
}

impl From<DeclId<ty::TyAbiDecl>> for InterfaceDeclId {
    fn from(id: DeclId<ty::TyAbiDecl>) -> Self {
        Self::Abi(id)
    }
}

impl From<DeclId<ty::TyTraitDecl>> for InterfaceDeclId {
    fn from(id: DeclId<ty::TyTraitDecl>) -> Self {
        Self::Trait(id)
    }
}
