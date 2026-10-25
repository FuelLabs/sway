use sway_error::error::CompileError;
use sway_types::{Named, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::{DebugWithEngines, DisplayWithEngines},
    language::ty::{self, TyFunctionDecl},
    Engines,
};

/// [FunctionalDeclId] exists to trace and replace function `DeclId`s,
/// e.g., in function applications.
///
/// In order for replacements to work:
/// - every dummy function must be linked to its `TraitFn`,
/// - every modified copy of a `TyFunctionDecl/TyTraitFn` must be linked to its original.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FunctionalDeclId {
    TraitFn(DeclId<ty::TyTraitFn>),
    Function(DeclId<ty::TyFunctionDecl>),
}

impl FunctionalDeclId {
    pub fn span(&self, engines: &Engines) -> Span {
        match self {
            Self::TraitFn(decl_id) => engines.de().get(decl_id).span(),
            Self::Function(decl_id) => engines.de().get(decl_id).span(),
        }
    }
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

impl DisplayWithEngines for FunctionalDeclId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        match self {
            Self::TraitFn(decl_id) => {
                write!(
                    f,
                    "decl(trait function {})",
                    engines.de().get(decl_id).name()
                )
            }
            Self::Function(decl_id) => {
                write!(f, "decl(function {})", engines.de().get(decl_id).name())
            }
        }
    }
}

impl DebugWithEngines for FunctionalDeclId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &Engines) -> std::fmt::Result {
        match self {
            Self::TraitFn(decl_id) => {
                let decl = engines.de().get(decl_id);
                write!(f, "decl(trait function {:?})", engines.help_out(decl))
            }
            Self::Function(decl_id) => {
                write!(
                    f,
                    "decl(function {:?})",
                    engines.help_out(engines.de().get(decl_id))
                )
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
        }
    }
}
impl TryFrom<&FunctionalDeclId> for DeclId<TyFunctionDecl> {
    type Error = CompileError;
    fn try_from(value: &FunctionalDeclId) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}
