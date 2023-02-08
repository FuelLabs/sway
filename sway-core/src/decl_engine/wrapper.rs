use std::{
    fmt,
    hash::{Hash, Hasher},
};

use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    engine_threading::*,
    language::ty,
    type_system::{SubstTypes, TypeSubstMap},
    ReplaceSelfType, TypeId,
};

use super::{DeclMapping, ReplaceDecls, ReplaceFunctionImplementingType};

/// The [DeclEngine] type is used in the [DeclarationEngine] as a means of
/// placing all declaration types into the same type.
#[derive(Clone, Debug)]
pub enum DeclWrapper {
    // no-op variant to fulfill the default trait
    Unknown,
    Function(ty::TyFunctionDeclaration),
    Trait(ty::TyTraitDeclaration),
    TraitFn(ty::TyTraitFn),
    ImplTrait(ty::TyImplTrait),
    Struct(ty::TyStructDeclaration),
    Storage(ty::TyStorageDeclaration),
    Abi(ty::TyAbiDeclaration),
    Constant(Box<ty::TyConstantDeclaration>),
    Enum(ty::TyEnumDeclaration),
}

impl Default for DeclWrapper {
    fn default() -> Self {
        DeclWrapper::Unknown
    }
}

impl PartialEqWithEngines for DeclWrapper {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (DeclWrapper::Unknown, DeclWrapper::Unknown) => true,
            (DeclWrapper::Function(l), DeclWrapper::Function(r)) => l.eq(r, engines),
            (DeclWrapper::Trait(l), DeclWrapper::Trait(r)) => l.eq(r, engines),
            (DeclWrapper::TraitFn(l), DeclWrapper::TraitFn(r)) => l.eq(r, engines),
            (DeclWrapper::ImplTrait(l), DeclWrapper::ImplTrait(r)) => l.eq(r, engines),
            (DeclWrapper::Struct(l), DeclWrapper::Struct(r)) => l.eq(r, engines),
            (DeclWrapper::Storage(l), DeclWrapper::Storage(r)) => l.eq(r, engines),
            (DeclWrapper::Abi(l), DeclWrapper::Abi(r)) => l.eq(r, engines),
            (DeclWrapper::Constant(l), DeclWrapper::Constant(r)) => l.eq(r, engines),
            (DeclWrapper::Enum(l), DeclWrapper::Enum(r)) => l.eq(r, engines),
            _ => false,
        }
    }
}

impl HashWithEngines for DeclWrapper {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        use DeclWrapper::*;
        std::mem::discriminant(self).hash(state);
        match self {
            Unknown => {}
            Function(decl) => {
                decl.hash(state, engines);
            }
            Trait(decl) => {
                decl.hash(state, engines);
            }
            TraitFn(decl) => {
                decl.hash(state, engines);
            }
            ImplTrait(decl) => {
                decl.hash(state, engines);
            }
            Struct(decl) => {
                decl.hash(state, engines);
            }
            Storage(decl) => {
                decl.hash(state, engines);
            }
            Abi(decl) => {
                decl.hash(state, engines);
            }
            Constant(decl) => {
                decl.hash(state, engines);
            }
            Enum(decl) => {
                decl.hash(state, engines);
            }
        }
    }
}

impl fmt::Display for DeclWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decl({})", self.friendly_name())
    }
}

impl SubstTypes for DeclWrapper {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        match self {
            DeclWrapper::Unknown => {}
            DeclWrapper::Function(decl) => decl.subst(type_mapping, engines),
            DeclWrapper::Trait(decl) => decl.subst(type_mapping, engines),
            DeclWrapper::TraitFn(decl) => decl.subst(type_mapping, engines),
            DeclWrapper::ImplTrait(decl) => decl.subst(type_mapping, engines),
            DeclWrapper::Struct(decl) => decl.subst(type_mapping, engines),
            DeclWrapper::Storage(_) => {}
            DeclWrapper::Abi(_) => {}
            DeclWrapper::Constant(_) => {}
            DeclWrapper::Enum(decl) => decl.subst(type_mapping, engines),
        }
    }
}

impl ReplaceSelfType for DeclWrapper {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        match self {
            DeclWrapper::Unknown => {}
            DeclWrapper::Function(decl) => decl.replace_self_type(engines, self_type),
            DeclWrapper::Trait(decl) => decl.replace_self_type(engines, self_type),
            DeclWrapper::TraitFn(decl) => decl.replace_self_type(engines, self_type),
            DeclWrapper::ImplTrait(decl) => decl.replace_self_type(engines, self_type),
            DeclWrapper::Struct(decl) => decl.replace_self_type(engines, self_type),
            DeclWrapper::Storage(_) => {}
            DeclWrapper::Abi(_) => {}
            DeclWrapper::Constant(_) => {}
            DeclWrapper::Enum(decl) => decl.replace_self_type(engines, self_type),
        }
    }
}

impl ReplaceDecls for DeclWrapper {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if let DeclWrapper::Function(decl) = self {
            decl.replace_decls(decl_mapping, engines);
        }
    }
}

impl ReplaceFunctionImplementingType for DeclWrapper {
    fn replace_implementing_type(
        &mut self,
        _engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    ) {
        match self {
            DeclWrapper::Function(decl) => decl.set_implementing_type(implementing_type),
            DeclWrapper::Unknown
            | DeclWrapper::Trait(_)
            | DeclWrapper::TraitFn(_)
            | DeclWrapper::ImplTrait(_)
            | DeclWrapper::Struct(_)
            | DeclWrapper::Storage(_)
            | DeclWrapper::Abi(_)
            | DeclWrapper::Constant(_)
            | DeclWrapper::Enum(_) => {}
        }
    }
}

impl From<ty::TyFunctionDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyFunctionDeclaration) -> Self {
        let span = value.span();
        (DeclWrapper::Function(value), span)
    }
}

impl From<ty::TyTraitDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyTraitDeclaration) -> Self {
        let span = value.name.span();
        (DeclWrapper::Trait(value), span)
    }
}

impl From<ty::TyTraitFn> for (DeclWrapper, Span) {
    fn from(value: ty::TyTraitFn) -> Self {
        let span = value.name.span();
        (DeclWrapper::TraitFn(value), span)
    }
}

impl From<ty::TyImplTrait> for (DeclWrapper, Span) {
    fn from(value: ty::TyImplTrait) -> Self {
        let span = value.span.clone();
        (DeclWrapper::ImplTrait(value), span)
    }
}

impl From<ty::TyStructDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyStructDeclaration) -> Self {
        let span = value.span();
        (DeclWrapper::Struct(value), span)
    }
}

impl From<ty::TyStorageDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyStorageDeclaration) -> Self {
        let span = value.span();
        (DeclWrapper::Storage(value), span)
    }
}

impl From<ty::TyAbiDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyAbiDeclaration) -> Self {
        let span = value.span.clone();
        (DeclWrapper::Abi(value), span)
    }
}

impl From<ty::TyConstantDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyConstantDeclaration) -> Self {
        let span = value.name.span();
        (DeclWrapper::Constant(Box::new(value)), span)
    }
}

impl From<ty::TyEnumDeclaration> for (DeclWrapper, Span) {
    fn from(value: ty::TyEnumDeclaration) -> Self {
        let span = value.span();
        (DeclWrapper::Enum(value), span)
    }
}

impl DeclWrapper {
    /// friendly name string used for error reporting.
    fn friendly_name(&self) -> &'static str {
        match self {
            DeclWrapper::Unknown => "unknown",
            DeclWrapper::Function(_) => "function",
            DeclWrapper::Trait(_) => "trait",
            DeclWrapper::Struct(_) => "struct",
            DeclWrapper::ImplTrait(_) => "impl trait",
            DeclWrapper::TraitFn(_) => "trait function",
            DeclWrapper::Storage(_) => "storage",
            DeclWrapper::Abi(_) => "abi",
            DeclWrapper::Constant(_) => "constant",
            DeclWrapper::Enum(_) => "enum",
        }
    }

    pub(super) fn expect_function(
        self,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        match self {
            DeclWrapper::Function(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotAFunction {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_trait(self, span: &Span) -> Result<ty::TyTraitDeclaration, CompileError> {
        match self {
            DeclWrapper::Trait(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotATrait {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_trait_fn(self, span: &Span) -> Result<ty::TyTraitFn, CompileError> {
        match self {
            DeclWrapper::TraitFn(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotATraitFn {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_impl_trait(self, span: &Span) -> Result<ty::TyImplTrait, CompileError> {
        match self {
            DeclWrapper::ImplTrait(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotAnImplTrait {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_struct(
        self,
        span: &Span,
    ) -> Result<ty::TyStructDeclaration, CompileError> {
        match self {
            DeclWrapper::Struct(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotAStruct {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_storage(
        self,
        span: &Span,
    ) -> Result<ty::TyStorageDeclaration, CompileError> {
        match self {
            DeclWrapper::Storage(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotStorage {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }

    pub(super) fn expect_abi(self, span: &Span) -> Result<ty::TyAbiDeclaration, CompileError> {
        match self {
            DeclWrapper::Abi(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            _ => Err(CompileError::Internal(
                "expected ABI definition",
                span.clone(),
            )),
        }
    }

    pub(super) fn expect_constant(
        self,
        span: &Span,
    ) -> Result<ty::TyConstantDeclaration, CompileError> {
        match self {
            DeclWrapper::Constant(decl) => Ok(*decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            _ => Err(CompileError::Internal(
                "expected to find constant definition",
                span.clone(),
            )),
        }
    }

    pub(super) fn expect_enum(self, span: &Span) -> Result<ty::TyEnumDeclaration, CompileError> {
        match self {
            DeclWrapper::Enum(decl) => Ok(decl),
            DeclWrapper::Unknown => Err(CompileError::Internal(
                "did not expect to find unknown declaration",
                span.clone(),
            )),
            actually => Err(CompileError::DeclIsNotAnEnum {
                actually: actually.friendly_name().to_string(),
                span: span.clone(),
            }),
        }
    }
}
