use std::{cmp::Ordering, hash::Hasher};

use crate::{decl_engine::DeclId, engine_threading::*, language::ty, type_system::*};

#[derive(Debug)]
pub(crate) enum Instruction {
    Type(TypeId, SubstList),
    FnDecl(DeclId<ty::TyFunctionDeclaration>, SubstList),
    TraitDecl(DeclId<ty::TyTraitDeclaration>, SubstList),
    ImplTrait(DeclId<ty::TyImplTrait>, SubstList),
    StructDecl(DeclId<ty::TyStructDeclaration>, SubstList),
    EnumDecl(DeclId<ty::TyEnumDeclaration>, SubstList),
}

impl Instruction {
    fn discriminant_value(&self) -> u8 {
        use Instruction::*;
        match self {
            Type(_, _) => 0,
            FnDecl(_, _) => 1,
            TraitDecl(_, _) => 2,
            ImplTrait(_, _) => 3,
            StructDecl(_, _) => 4,
            EnumDecl(_, _) => 5,
        }
    }
}

impl EqWithEngines for Instruction {}
impl PartialEqWithEngines for Instruction {
    fn eq(&self, other: &Self, _engines: Engines<'_>) -> bool {
        use Instruction::*;
        match (self, other) {
            (Type(_, _), Type(_, _)) => todo!(),
            (FnDecl(_, _), FnDecl(_, _)) => todo!(),
            (TraitDecl(_, _), TraitDecl(_, _)) => todo!(),
            (ImplTrait(_, _), ImplTrait(_, _)) => todo!(),
            (StructDecl(_, _), StructDecl(_, _)) => todo!(),
            (EnumDecl(_, _), EnumDecl(_, _)) => todo!(),
            (l, r) => l.discriminant_value() == r.discriminant_value(),
        }
    }
}

impl HashWithEngines for Instruction {
    fn hash<H: Hasher>(&self, _state: &mut H, _engines: Engines<'_>) {
        use Instruction::*;
        match self {
            Type(_, _) => todo!(),
            FnDecl(_, _) => todo!(),
            TraitDecl(_, _) => todo!(),
            ImplTrait(_, _) => todo!(),
            StructDecl(_, _) => todo!(),
            EnumDecl(_, _) => todo!(),
        }
    }
}

impl OrdWithEngines for Instruction {
    fn cmp(&self, other: &Self, _engines: Engines<'_>) -> Ordering {
        use Instruction::*;
        match (self, other) {
            (Type(_, _), Type(_, _)) => todo!(),
            (FnDecl(_, _), FnDecl(_, _)) => todo!(),
            (TraitDecl(_, _), TraitDecl(_, _)) => todo!(),
            (ImplTrait(_, _), ImplTrait(_, _)) => todo!(),
            (StructDecl(_, _), StructDecl(_, _)) => todo!(),
            (EnumDecl(_, _), EnumDecl(_, _)) => todo!(),
            (l, r) => l.discriminant_value().cmp(&r.discriminant_value()),
        }
    }
}
