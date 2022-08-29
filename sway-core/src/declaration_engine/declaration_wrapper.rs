use crate::{
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    TypedFunctionDeclaration,
};

/// The [DeclarationWrapper] type is used in the [DeclarationEngine]
/// as a means of placing all declaration types into the same type.
#[derive(Clone)]
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
    pub(super) fn expect_function(self) -> Result<TypedFunctionDeclaration, String> {
        match self {
            DeclarationWrapper::Function(decl) => Ok(decl),
            _ => Err("expected to find function declaration".to_string()),
        }
    }

    pub(super) fn expect_trait(self) -> Result<TypedTraitDeclaration, String> {
        match self {
            DeclarationWrapper::Trait(decl) => Ok(decl),
            _ => Err("expected to find trait declaration".to_string()),
        }
    }

    pub(super) fn expect_trait_fn(self) -> Result<TypedTraitFn, String> {
        match self {
            DeclarationWrapper::TraitFn(decl) => Ok(decl),
            _ => Err("expected to find trait fn".to_string()),
        }
    }

    pub(super) fn expect_trait_impl(self) -> Result<TypedImplTrait, String> {
        match self {
            DeclarationWrapper::TraitImpl(decl) => Ok(decl),
            _ => Err("expected to find trait impl".to_string()),
        }
    }

    pub(super) fn expect_struct(self) -> Result<TypedStructDeclaration, String> {
        match self {
            DeclarationWrapper::Struct(decl) => Ok(decl),
            _ => Err("expected to find struct declaration".to_string()),
        }
    }
}
