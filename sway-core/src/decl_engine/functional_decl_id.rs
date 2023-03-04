use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    decl_engine::*,
    language::ty::{self, TyFunctionDeclaration},
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FunctionalDeclId {
    TraitFn(DeclId<ty::TyTraitFn>),
    Function(DeclId<ty::TyFunctionDeclaration>),
}

impl From<DeclId<ty::TyFunctionDeclaration>> for FunctionalDeclId {
    fn from(val: DeclId<ty::TyFunctionDeclaration>) -> Self {
        Self::Function(val)
    }
}
impl From<&DeclId<ty::TyFunctionDeclaration>> for FunctionalDeclId {
    fn from(val: &DeclId<ty::TyFunctionDeclaration>) -> Self {
        Self::Function(*val)
    }
}
impl From<&mut DeclId<ty::TyFunctionDeclaration>> for FunctionalDeclId {
    fn from(val: &mut DeclId<ty::TyFunctionDeclaration>) -> Self {
        Self::Function(*val)
    }
}

impl From<DeclId<ty::TyTraitFn>> for FunctionalDeclId {
    fn from(val: DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(val)
    }
}
impl From<&DeclId<ty::TyTraitFn>> for FunctionalDeclId {
    fn from(val: &DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(*val)
    }
}
impl From<&mut DeclId<ty::TyTraitFn>> for FunctionalDeclId {
    fn from(val: &mut DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(*val)
    }
}

impl std::fmt::Display for FunctionalDeclId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TraitFn(_) => {
                write!(f, "decl(trait function)",)
            }
            Self::Function(_) => {
                write!(f, "decl(function)",)
            }
        }
    }
}

impl TryFrom<DeclRefMixedFunctional> for DeclRefFunction {
    type Error = CompileError;
    fn try_from(value: DeclRefMixedFunctional) -> Result<Self, Self::Error> {
        match value.id {
            FunctionalDeclId::Function(id) => Ok(DeclRef {
                name: value.name,
                id,
                decl_span: value.decl_span,
            }),
            actually @ FunctionalDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: value.decl_span,
            }),
        }
    }
}
impl TryFrom<&DeclRefMixedFunctional> for DeclRefFunction {
    type Error = CompileError;
    fn try_from(value: &DeclRefMixedFunctional) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}

impl TryFrom<FunctionalDeclId> for DeclId<TyFunctionDeclaration> {
    type Error = CompileError;
    fn try_from(value: FunctionalDeclId) -> Result<Self, Self::Error> {
        match value {
            FunctionalDeclId::Function(id) => Ok(id),
            actually @ FunctionalDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
        }
    }
}
impl TryFrom<&FunctionalDeclId> for DeclId<TyFunctionDeclaration> {
    type Error = CompileError;
    fn try_from(value: &FunctionalDeclId) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}
