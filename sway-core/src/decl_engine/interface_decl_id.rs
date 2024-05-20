use crate::{
    decl_engine::*,
    language::{
        parsed::{AbiDeclaration, TraitDeclaration},
        ty,
    },
};

use super::parsed_id::ParsedDeclId;

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum ParsedInterfaceDeclId {
    Abi(ParsedDeclId<AbiDeclaration>),
    Trait(ParsedDeclId<TraitDeclaration>),
}

impl From<ParsedDeclId<AbiDeclaration>> for ParsedInterfaceDeclId {
    fn from(id: ParsedDeclId<AbiDeclaration>) -> Self {
        Self::Abi(id)
    }
}

impl From<ParsedDeclId<TraitDeclaration>> for ParsedInterfaceDeclId {
    fn from(id: ParsedDeclId<TraitDeclaration>) -> Self {
        Self::Trait(id)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
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
