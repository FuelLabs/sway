use std::fmt;

use sway_types::Span;

use crate::{
    semantic_analysis::{
        TypedImplTrait, TypedStructDeclaration, TypedTraitDeclaration, TypedTraitFn,
    },
    CompileError, TypedFunctionDeclaration,
};

/// The [DeclarationWrapper] type is used in the [DeclarationEngine]
/// as a means of placing all declaration types into the same type.
#[derive(Clone, Debug)]
pub(crate) enum DeclarationWrapper {
    // no-op variant to fulfill the default trait
    Unknown,
    Function(TypedFunctionDeclaration),
    Trait(TypedTraitDeclaration),
    TraitFn(TypedTraitFn),
    TraitImpl(TypedImplTrait),
    Struct(TypedStructDeclaration),
}

impl Default for DeclarationWrapper {
    fn default() -> Self {
        DeclarationWrapper::Unknown
    }
}

impl PartialEq for DeclarationWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DeclarationWrapper::Unknown, DeclarationWrapper::Unknown) => true,
            (DeclarationWrapper::Function(l), DeclarationWrapper::Function(r)) => l == r,
            (DeclarationWrapper::Trait(l), DeclarationWrapper::Trait(r)) => l == r,
            (DeclarationWrapper::TraitFn(l), DeclarationWrapper::TraitFn(r)) => l == r,
            (DeclarationWrapper::TraitImpl(l), DeclarationWrapper::TraitImpl(r)) => l == r,
            (DeclarationWrapper::Struct(l), DeclarationWrapper::Struct(r)) => l == r,
            _ => false,
        }
    }
}

impl fmt::Display for DeclarationWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "decl({})", self.friendly_name())
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
            DeclarationWrapper::TraitImpl(_) => "impl trait",
            DeclarationWrapper::TraitFn(_) => "trait function",
        }
    }

    pub(super) fn expect_function(
        self,
        span: &Span,
    ) -> Result<TypedFunctionDeclaration, CompileError> {
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

    pub(super) fn expect_trait(self, span: &Span) -> Result<TypedTraitDeclaration, CompileError> {
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

    pub(super) fn expect_trait_fn(self, span: &Span) -> Result<TypedTraitFn, CompileError> {
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

    pub(super) fn expect_trait_impl(self, span: &Span) -> Result<TypedImplTrait, CompileError> {
        match self {
            DeclarationWrapper::TraitImpl(decl) => Ok(decl),
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

    pub(super) fn expect_struct(self, span: &Span) -> Result<TypedStructDeclaration, CompileError> {
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
}
