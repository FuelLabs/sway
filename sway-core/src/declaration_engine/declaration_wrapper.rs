use std::fmt;

use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    language::ty::{self, TyDeclaration},
    type_system::{CopyTypes, TypeMapping},
    PartialEqWithTypeEngine, ReplaceSelfType, TypeEngine, TypeId,
};

use super::{DeclMapping, ReplaceDecls, ReplaceFunctionImplementingType};

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
impl PartialEqWithTypeEngine for DeclarationWrapper {
    fn eq(&self, other: &Self, type_engine: &TypeEngine) -> bool {
        match (self, other) {
            (DeclarationWrapper::Unknown, DeclarationWrapper::Unknown) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => {
                l.eq(r, type_engine)
            }
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => l.eq(r, type_engine),
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => {
                l.eq(r, type_engine)
            }
            (DeclarationWrapper::ImplTrait(l), DeclarationWrapper::ImplTrait(r)) => {
                l.eq(r, type_engine)
            }
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => l.eq(r, type_engine),
            (DeclarationWrapper::Storage(l), DeclarationWrapper::Storage(r)) => {
                l.eq(r, type_engine)
            }
            (DeclarationWrapper::Abi(l), DeclarationWrapper::Abi(r)) => l.eq(r, type_engine),
            (DeclarationWrapper::Constant(l), DeclarationWrapper::Constant(r)) => {
                l.eq(r, type_engine)
            }
            (DeclarationWrapper::Enum(l), DeclarationWrapper::Enum(r)) => l.eq(r, type_engine),
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
    fn copy_types_inner(&mut self, type_mapping: &TypeMapping, type_engine: &TypeEngine) {
        match self {
            DeclarationWrapper::Unknown => {}
            DeclarationWrapper::Function(decl) => decl.copy_types(type_mapping, type_engine),
            DeclarationWrapper::Trait(decl) => decl.copy_types(type_mapping, type_engine),
            DeclarationWrapper::TraitFn(decl) => decl.copy_types(type_mapping, type_engine),
            DeclarationWrapper::ImplTrait(decl) => decl.copy_types(type_mapping, type_engine),
            DeclarationWrapper::Struct(decl) => decl.copy_types(type_mapping, type_engine),
            DeclarationWrapper::Storage(_) => {}
            DeclarationWrapper::Abi(_) => {}
            DeclarationWrapper::Constant(_) => {}
            DeclarationWrapper::Enum(decl) => decl.copy_types(type_mapping, type_engine),
        }
    }
}

impl ReplaceSelfType for DeclarationWrapper {
    fn replace_self_type(&mut self, type_engine: &TypeEngine, self_type: TypeId) {
        match self {
            DeclarationWrapper::Unknown => {}
            DeclarationWrapper::Function(decl) => decl.replace_self_type(type_engine, self_type),
            DeclarationWrapper::Trait(decl) => decl.replace_self_type(type_engine, self_type),
            DeclarationWrapper::TraitFn(decl) => decl.replace_self_type(type_engine, self_type),
            DeclarationWrapper::ImplTrait(decl) => decl.replace_self_type(type_engine, self_type),
            DeclarationWrapper::Struct(decl) => decl.replace_self_type(type_engine, self_type),
            DeclarationWrapper::Storage(_) => {}
            DeclarationWrapper::Abi(_) => {}
            DeclarationWrapper::Constant(_) => {}
            DeclarationWrapper::Enum(decl) => decl.replace_self_type(type_engine, self_type),
        }
    }
}

impl ReplaceDecls for DeclarationWrapper {
    fn replace_decls_inner(&mut self, decl_mapping: &DeclMapping, type_engine: &TypeEngine) {
        if let DeclarationWrapper::Function(decl) = self {
            decl.replace_decls(decl_mapping, type_engine);
        }
    }
}

impl ReplaceFunctionImplementingType for DeclarationWrapper {
    fn replace_implementing_type(&mut self, implementing_type: TyDeclaration) {
        match self {
            DeclarationWrapper::Function(decl) => decl.set_implementing_type(implementing_type),
            DeclarationWrapper::Unknown
            | DeclarationWrapper::Trait(_)
            | DeclarationWrapper::TraitFn(_)
            | DeclarationWrapper::ImplTrait(_)
            | DeclarationWrapper::Struct(_)
            | DeclarationWrapper::Storage(_)
            | DeclarationWrapper::Abi(_)
            | DeclarationWrapper::Constant(_)
            | DeclarationWrapper::Enum(_) => {}
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
