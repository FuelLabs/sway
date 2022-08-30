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
