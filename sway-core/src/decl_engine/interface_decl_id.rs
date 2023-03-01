use crate::{decl_engine::*, language::ty};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum InterfaceDeclId {
    Abi(DeclId<ty::TyAbiDeclaration>),
    Trait(DeclId<ty::TyTraitDeclaration>),
}

impl From<DeclId<ty::TyAbiDeclaration>> for InterfaceDeclId {
    fn from(id: DeclId<ty::TyAbiDeclaration>) -> Self {
        Self::Abi(id)
    }
}

impl From<DeclId<ty::TyTraitDeclaration>> for InterfaceDeclId {
    fn from(id: DeclId<ty::TyTraitDeclaration>) -> Self {
        Self::Trait(id)
    }
}
