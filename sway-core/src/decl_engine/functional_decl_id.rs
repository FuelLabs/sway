use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    decl_engine::*,
    language::ty::{self, TyFunctionDecl},
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FunctionalDeclId {
    TraitFn(DeclId<ty::TyTraitFn>),
    Function(DeclId<ty::TyFunctionDecl>),
    Constant(DeclId<ty::TyConstantDecl>),
}

impl From<DeclId<ty::TyFunctionDecl>> for FunctionalDeclId {
    fn from(val: DeclId<ty::TyFunctionDecl>) -> Self {
        Self::Function(val)
    }
}
impl From<&DeclId<ty::TyFunctionDecl>> for FunctionalDeclId {
    fn from(val: &DeclId<ty::TyFunctionDecl>) -> Self {
        Self::Function(*val)
    }
}
impl From<&mut DeclId<ty::TyFunctionDecl>> for FunctionalDeclId {
    fn from(val: &mut DeclId<ty::TyFunctionDecl>) -> Self {
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

impl From<DeclId<ty::TyConstantDecl>> for FunctionalDeclId {
    fn from(val: DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(val)
    }
}
impl From<&DeclId<ty::TyConstantDecl>> for FunctionalDeclId {
    fn from(val: &DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(*val)
    }
}
impl From<&mut DeclId<ty::TyConstantDecl>> for FunctionalDeclId {
    fn from(val: &mut DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(*val)
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
            Self::Constant(_) => {
                write!(f, "decl(constant)",)
            }
        }
    }
}

impl TryFrom<DeclRefMixedFunctional> for DeclRefFunction {
    type Error = CompileError;
    fn try_from(value: DeclRefMixedFunctional) -> Result<Self, Self::Error> {
        match value.id().clone() {
            FunctionalDeclId::Function(id) => Ok(DeclRef::new(
                value.name().clone(),
                id,
                value.decl_span().clone(),
            )),
            actually @ FunctionalDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: value.decl_span().clone(),
            }),
            actually @ FunctionalDeclId::Constant(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: value.decl_span().clone(),
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

impl TryFrom<FunctionalDeclId> for DeclId<TyFunctionDecl> {
    type Error = CompileError;
    fn try_from(value: FunctionalDeclId) -> Result<Self, Self::Error> {
        match value {
            FunctionalDeclId::Function(id) => Ok(id),
            actually @ FunctionalDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
            actually @ FunctionalDeclId::Constant(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
        }
    }
}
impl TryFrom<&FunctionalDeclId> for DeclId<TyFunctionDecl> {
    type Error = CompileError;
    fn try_from(value: &FunctionalDeclId) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}
