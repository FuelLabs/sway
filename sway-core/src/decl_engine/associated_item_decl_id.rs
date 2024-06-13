use sway_error::error::CompileError;
use sway_types::{Named, Span, Spanned};

use crate::{
    decl_engine::*,
    engine_threading::{DebugWithEngines, DisplayWithEngines},
    language::ty::{self, TyFunctionDecl},
    Engines,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum AssociatedItemDeclId {
    TraitFn(DeclId<ty::TyTraitFn>),
    Function(DeclId<ty::TyFunctionDecl>),
    Constant(DeclId<ty::TyConstantDecl>),
    Type(DeclId<ty::TyTraitType>),
}

impl AssociatedItemDeclId {
    pub fn span(&self, engines: &Engines) -> Span {
        match self {
            Self::TraitFn(decl_id) => engines.de().get(decl_id).span(),
            Self::Function(decl_id) => engines.de().get(decl_id).span(),
            Self::Constant(decl_id) => engines.de().get(decl_id).span(),
            Self::Type(decl_id) => engines.de().get(decl_id).span(),
        }
    }
}

impl From<DeclId<ty::TyFunctionDecl>> for AssociatedItemDeclId {
    fn from(val: DeclId<ty::TyFunctionDecl>) -> Self {
        Self::Function(val)
    }
}
impl From<&DeclId<ty::TyFunctionDecl>> for AssociatedItemDeclId {
    fn from(val: &DeclId<ty::TyFunctionDecl>) -> Self {
        Self::Function(*val)
    }
}
impl From<&mut DeclId<ty::TyFunctionDecl>> for AssociatedItemDeclId {
    fn from(val: &mut DeclId<ty::TyFunctionDecl>) -> Self {
        Self::Function(*val)
    }
}

impl From<DeclId<ty::TyTraitFn>> for AssociatedItemDeclId {
    fn from(val: DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(val)
    }
}
impl From<&DeclId<ty::TyTraitFn>> for AssociatedItemDeclId {
    fn from(val: &DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(*val)
    }
}
impl From<&mut DeclId<ty::TyTraitFn>> for AssociatedItemDeclId {
    fn from(val: &mut DeclId<ty::TyTraitFn>) -> Self {
        Self::TraitFn(*val)
    }
}

impl From<DeclId<ty::TyTraitType>> for AssociatedItemDeclId {
    fn from(val: DeclId<ty::TyTraitType>) -> Self {
        Self::Type(val)
    }
}
impl From<&DeclId<ty::TyTraitType>> for AssociatedItemDeclId {
    fn from(val: &DeclId<ty::TyTraitType>) -> Self {
        Self::Type(*val)
    }
}
impl From<&mut DeclId<ty::TyTraitType>> for AssociatedItemDeclId {
    fn from(val: &mut DeclId<ty::TyTraitType>) -> Self {
        Self::Type(*val)
    }
}

impl From<DeclId<ty::TyConstantDecl>> for AssociatedItemDeclId {
    fn from(val: DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(val)
    }
}
impl From<&DeclId<ty::TyConstantDecl>> for AssociatedItemDeclId {
    fn from(val: &DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(*val)
    }
}
impl From<&mut DeclId<ty::TyConstantDecl>> for AssociatedItemDeclId {
    fn from(val: &mut DeclId<ty::TyConstantDecl>) -> Self {
        Self::Constant(*val)
    }
}

impl std::fmt::Display for AssociatedItemDeclId {
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
            Self::Type(_) => {
                write!(f, "decl(type)",)
            }
        }
    }
}

impl DisplayWithEngines for AssociatedItemDeclId {
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
            Self::Constant(decl_id) => {
                write!(f, "decl(constant {})", engines.de().get(decl_id).name())
            }
            Self::Type(decl_id) => {
                write!(f, "decl(type {})", engines.de().get(decl_id).name())
            }
        }
    }
}

impl DebugWithEngines for AssociatedItemDeclId {
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
            Self::Constant(decl_id) => {
                write!(
                    f,
                    "decl(constant {:?})",
                    engines.help_out(engines.de().get(decl_id))
                )
            }
            Self::Type(decl_id) => {
                write!(
                    f,
                    "decl(type {:?})",
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
            AssociatedItemDeclId::Function(id) => Ok(DeclRef::new(
                value.name().clone(),
                id,
                value.decl_span().clone(),
            )),
            actually @ AssociatedItemDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: value.decl_span().clone(),
            }),
            actually @ AssociatedItemDeclId::Constant(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: value.decl_span().clone(),
            }),
            actually @ AssociatedItemDeclId::Type(_) => Err(CompileError::DeclIsNotAFunction {
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

impl TryFrom<AssociatedItemDeclId> for DeclId<TyFunctionDecl> {
    type Error = CompileError;
    fn try_from(value: AssociatedItemDeclId) -> Result<Self, Self::Error> {
        match value {
            AssociatedItemDeclId::Function(id) => Ok(id),
            actually @ AssociatedItemDeclId::TraitFn(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
            actually @ AssociatedItemDeclId::Constant(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
            actually @ AssociatedItemDeclId::Type(_) => Err(CompileError::DeclIsNotAFunction {
                actually: actually.to_string(),
                span: Span::dummy(), // FIXME
            }),
        }
    }
}
impl TryFrom<&AssociatedItemDeclId> for DeclId<TyFunctionDecl> {
    type Error = CompileError;
    fn try_from(value: &AssociatedItemDeclId) -> Result<Self, Self::Error> {
        value.clone().try_into()
    }
}
