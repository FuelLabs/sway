use std::fmt;

use crate::{
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    TypedFunctionDeclaration,
};

/// The [DeclarationWrapper] type is used in the [DeclarationEngine]
/// as a means of placing all declaration types into the same type.
#[derive(Clone, Debug)]
pub(crate) enum DeclarationWrapper {
    // no-op variant to fulfill the default trait
    Default,
    Function(TypedFunctionDeclaration),
    Trait(TypedTraitDeclaration),
    TraitFn(TypedTraitFn),
    TraitImpl(TypedImplTrait),
    Struct(TypedStructDeclaration),
}

impl Default for DeclarationWrapper {
    fn default() -> Self {
        DeclarationWrapper::Default
    }
}

impl PartialEq for DeclarationWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DeclarationWrapper::Default, DeclarationWrapper::Default) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => l == r,
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => l == r,
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => l == r,
            (DeclarationWrapper::TraitImpl(l), DeclarationWrapper::TraitImpl(r)) => l == r,
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => l == r,
            _ => false,
        }
    }
}

impl fmt::Display for DeclarationWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeclarationWrapper::Default => write!(f, "decl(DEFAULT)"),
            DeclarationWrapper::Function(_) => write!(f, "decl(function)"),
            DeclarationWrapper::Trait(_) => write!(f, "decl(trait)"),
            DeclarationWrapper::TraitFn(_) => write!(f, "decl(trait fn)"),
            DeclarationWrapper::TraitImpl(_) => write!(f, "decl(trait impl)"),
            DeclarationWrapper::Struct(_) => write!(f, "decl(decl)"),
        }
    }
}

impl DeclarationWrapper {
    pub(super) fn expect_function(self) -> Result<TypedFunctionDeclaration, DeclarationWrapper> {
        match self {
            DeclarationWrapper::Function(decl) => Ok(decl),
            actually => Err(actually),
        }
    }

    pub(super) fn expect_trait(self) -> Result<TypedTraitDeclaration, DeclarationWrapper> {
        match self {
            DeclarationWrapper::Trait(decl) => Ok(decl),
            actually => Err(actually),
        }
    }

    pub(super) fn expect_trait_fn(self) -> Result<TypedTraitFn, DeclarationWrapper> {
        match self {
            DeclarationWrapper::TraitFn(decl) => Ok(decl),
            actually => Err(actually),
        }
    }

    pub(super) fn expect_trait_impl(self) -> Result<TypedImplTrait, DeclarationWrapper> {
        match self {
            DeclarationWrapper::TraitImpl(decl) => Ok(decl),
            actually => Err(actually),
        }
    }

    pub(super) fn expect_struct(self) -> Result<TypedStructDeclaration, DeclarationWrapper> {
        match self {
            DeclarationWrapper::Struct(decl) => Ok(decl),
            actually => Err(actually),
        }
    }
}
