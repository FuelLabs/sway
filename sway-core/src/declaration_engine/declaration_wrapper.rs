use std::fmt;

use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    engine_threading::*,
    language::ty,
    type_system::{CopyTypes, TypeMapping},
    ReplaceSelfType, TypeId,
};

use super::{DeclMapping, ReplaceDecls};

/// The [DeclarationWrapper] type is used in the [DeclarationEngine]
/// as a means of placing all declaration types into the same type.
#[derive(Clone, Debug)]
pub enum DeclarationWrapper {
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

impl Default for DeclarationWrapper {
    fn default() -> Self {
        DeclarationWrapper::Unknown
    }
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEqWithEngines for DeclarationWrapper {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (DeclarationWrapper::Unknown, DeclarationWrapper::Unknown) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => l.eq(r, engines),
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => l.eq(r, engines),
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => l.eq(r, engines),
            (DeclarationWrapper::ImplTrait(l), DeclarationWrapper::ImplTrait(r)) => {
                l.eq(r, engines)
            }
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => l.eq(r, engines),
            (DeclarationWrapper::Storage(l), DeclarationWrapper::Storage(r)) => l.eq(r, engines),
            (DeclarationWrapper::Abi(l), DeclarationWrapper::Abi(r)) => l.eq(r, engines),
            (DeclarationWrapper::Constant(l), DeclarationWrapper::Constant(r)) => l.eq(r, engines),
            (DeclarationWrapper::Enum(l), DeclarationWrapper::Enum(r)) => l.eq(r, engines),
            _ => false,
        }
    }
}

impl fmt::Display for DeclarationWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decl({})", self.friendly_name())
    }
}

impl CopyTypes for DeclarationWrapper {
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, engines: Engines<'_>) {
        match self {
            DeclarationWrapper::Unknown => {}
            DeclarationWrapper::Function(decl) => decl.copy_types(type_mapping, engines),
            DeclarationWrapper::Trait(decl) => decl.copy_types(type_mapping, engines),
            DeclarationWrapper::TraitFn(decl) => decl.copy_types(type_mapping, engines),
            DeclarationWrapper::ImplTrait(decl) => decl.copy_types(type_mapping, engines),
            DeclarationWrapper::Struct(decl) => decl.copy_types(type_mapping, engines),
            DeclarationWrapper::Storage(_) => {}
            DeclarationWrapper::Abi(_) => {}
            DeclarationWrapper::Constant(_) => {}
            DeclarationWrapper::Enum(decl) => decl.copy_types(type_mapping, engines),
        }
    }
}

impl ReplaceSelfType for DeclarationWrapper {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        match self {
            DeclarationWrapper::Unknown => {}
            DeclarationWrapper::Function(decl) => decl.replace_self_type(engines, self_type),
            DeclarationWrapper::Trait(decl) => decl.replace_self_type(engines, self_type),
            DeclarationWrapper::TraitFn(decl) => decl.replace_self_type(engines, self_type),
            DeclarationWrapper::ImplTrait(decl) => decl.replace_self_type(engines, self_type),
            DeclarationWrapper::Struct(decl) => decl.replace_self_type(engines, self_type),
            DeclarationWrapper::Storage(_) => {}
            DeclarationWrapper::Abi(_) => {}
            DeclarationWrapper::Constant(_) => {}
            DeclarationWrapper::Enum(decl) => decl.replace_self_type(engines, self_type),
        }
    }
}

impl ReplaceDecls for DeclarationWrapper {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, engines: Engines<'_>) {
        if let DeclarationWrapper::Function(decl) = self {
            decl.replace_decls(decl_mapping, engines);
        }
    }
}

impl DeclarationWrapper {
    /// friendly name string used for error reporting.
    fn friendly_name(&self) -> &'static str {
        match self {
            DeclarationWrapper::Unknown => "unknown",
            DeclarationWrapper::Function(_) => "function",
            DeclarationWrapper::Trait(_) => "trait",
            DeclarationWrapper::Struct(_) => "struct",
            DeclarationWrapper::ImplTrait(_) => "impl trait",
            DeclarationWrapper::TraitFn(_) => "trait function",
            DeclarationWrapper::Storage(_) => "storage",
            DeclarationWrapper::Abi(_) => "abi",
            DeclarationWrapper::Constant(_) => "constant",
            DeclarationWrapper::Enum(_) => "enum",
        }
    }

    pub(super) fn expect_function(
        self,
        span: &Span,
    ) -> Result<ty::TyFunctionDeclaration, CompileError> {
        match self {
            DeclarationWrapper::Function(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Trait(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::TraitFn(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::ImplTrait(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Struct(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Storage(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Abi(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Constant(decl) => Ok(*decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
            DeclarationWrapper::Enum(decl) => Ok(decl),
            DeclarationWrapper::Unknown => Err(CompileError::Internal(
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
