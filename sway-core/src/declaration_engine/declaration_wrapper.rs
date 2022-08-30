use std::borrow::Borrow;

use crate::{
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    types::{CompileWrapper, ToCompileWrapper},
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

impl PartialEq for CompileWrapper<'_, DeclarationWrapper> {
    fn eq(&self, other: &Self) -> bool {
        let CompileWrapper {
            inner: me,
            declaration_engine: de,
        } = self;
        let CompileWrapper { inner: them, .. } = other;
        match (me.borrow(), them.borrow()) {
            (DeclarationWrapper::Default, DeclarationWrapper::Default) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => {
                l.wrap_ref(de) == r.wrap_ref(de)
            }
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => {
                l.wrap_ref(de) == r.wrap_ref(de)
            }
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => {
                l.wrap_ref(de) == r.wrap_ref(de)
            }
            (DeclarationWrapper::TraitImpl(l), DeclarationWrapper::TraitImpl(r)) => {
                l.wrap_ref(de) == r.wrap_ref(de)
            }
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => {
                l.wrap_ref(de) == r.wrap_ref(de)
            }
            _ => false,
        }
    }
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
