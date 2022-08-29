use crate::{
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    types::{CompileWrapper, ToCompileWrapper},
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

impl PartialEq for CompileWrapper<'_, DeclarationWrapper> {
    fn eq(&self, other: &Self) -> bool {
        match (self.inner, other.inner) {
            (DeclarationWrapper::Default, DeclarationWrapper::Default) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => {
                l.wrap(self.declaration_engine) == r.wrap(self.declaration_engine)
            }
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => l == r,
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => l == r,
            (DeclarationWrapper::TraitImpl(l), DeclarationWrapper::TraitImpl(r)) => {
                l.wrap(self.declaration_engine) == r.wrap(self.declaration_engine)
            }
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => {
                l.wrap(self.declaration_engine) == r.wrap(self.declaration_engine)
            }
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
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
