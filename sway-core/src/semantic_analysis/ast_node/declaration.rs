mod abi;
mod r#enum;
mod function;
mod impl_trait;
mod storage;
mod r#struct;
mod r#trait;
mod variable;

pub use abi::*;
pub use function::*;
pub use impl_trait::*;
pub use r#enum::*;
pub use r#struct::*;
pub use r#trait::*;
pub use storage::*;
pub use variable::*;

use crate::{
    declaration_engine::declaration_engine::*,
    error::*,
    language::{ty, *},
    semantic_analysis::*,
    type_system::*,
    AttributesMap,
};
use derivative::Derivative;
use std::borrow::Cow;
use sway_error::error::CompileError;
use sway_types::{Ident, Span, Spanned};

impl ty::TyDeclaration {
    /// Retrieves the declaration as an enum declaration.
    ///
    /// Returns an error if `self` is not a [TyEnumDeclaration].
    pub(crate) fn expect_enum(&self, access_span: &Span) -> CompileResult<TyEnumDeclaration> {
        match self {
            ty::TyDeclaration::EnumDeclaration(decl_id) => {
                CompileResult::from(de_get_enum(decl_id.clone(), access_span))
            }
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// Retrieves the declaration as a struct declaration.
    ///
    /// Returns an error if `self` is not a [TyStructDeclaration].
    pub(crate) fn expect_struct(&self, access_span: &Span) -> CompileResult<TyStructDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match self {
            ty::TyDeclaration::StructDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ok(decl, warnings, errors)
            }
            decl => {
                errors.push(CompileError::DeclIsNotAStruct {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a function declaration.
    ///
    /// Returns an error if `self` is not a [TyFunctionDeclaration].
    pub(crate) fn expect_function(
        &self,
        access_span: &Span,
    ) -> CompileResult<TyFunctionDeclaration> {
        let mut warnings = vec![];
        let mut errors = vec![];
        match self {
            ty::TyDeclaration::FunctionDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors,
                );
                ok(decl, warnings, errors)
            }
            decl => {
                errors.push(CompileError::DeclIsNotAFunction {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a variable declaration.
    ///
    /// Returns an error if `self` is not a [TyVariableDeclaration].
    pub(crate) fn expect_variable(&self) -> CompileResult<&TyVariableDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            ty::TyDeclaration::VariableDeclaration(decl) => ok(decl, warnings, errors),
            decl => {
                errors.push(CompileError::DeclIsNotAVariable {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as an Abi declaration.
    ///
    /// Returns an error if `self` is not a [ty::TyAbiDeclaration].
    pub(crate) fn expect_abi(&self, access_span: &Span) -> CompileResult<ty::TyAbiDeclaration> {
        match self {
            ty::TyDeclaration::AbiDeclaration(decl_id) => {
                CompileResult::from(de_get_abi(decl_id.clone(), access_span))
            }
            decl => err(
                vec![],
                vec![CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                }],
            ),
        }
    }

    /// friendly name string used for error reporting.
    pub fn friendly_name(&self) -> &'static str {
        use ty::TyDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery => "error",
            StorageDeclaration(_) => "contract storage declaration",
        }
    }

    pub(crate) fn return_type(&self, access_span: &Span) -> CompileResult<TypeId> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let type_id = match self {
            ty::TyDeclaration::VariableDeclaration(decl) => decl.body.return_type,
            ty::TyDeclaration::FunctionDeclaration { .. } => {
                errors.push(CompileError::Unimplemented(
                    "Function pointers have not yet been implemented.",
                    self.span(),
                ));
                return err(warnings, errors);
            }
            ty::TyDeclaration::StructDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.create_type_id()
            }
            ty::TyDeclaration::EnumDeclaration(decl_id) => {
                let decl = check!(
                    CompileResult::from(de_get_enum(decl_id.clone(), access_span)),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl.create_type_id()
            }
            ty::TyDeclaration::StorageDeclaration(decl_id) => {
                let storage_decl = check!(
                    CompileResult::from(de_get_storage(decl_id.clone(), &self.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                insert_type(TypeInfo::Storage {
                    fields: storage_decl.fields_as_typed_struct_fields(),
                })
            }
            ty::TyDeclaration::GenericTypeForFunctionScope { type_id, .. } => *type_id,
            decl => {
                errors.push(CompileError::NotAType {
                    span: decl.span(),
                    name: decl.to_string(),
                    actually_is: decl.friendly_name(),
                });
                return err(warnings, errors);
            }
        };
        ok(type_id, warnings, errors)
    }

    pub(crate) fn visibility(&self) -> CompileResult<Visibility> {
        use ty::TyDeclaration::*;
        let mut warnings = vec![];
        let mut errors = vec![];
        let visibility = match self {
            TraitDeclaration(decl_id) => {
                let TyTraitDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_trait(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            ConstantDeclaration(decl_id) => {
                let TyConstantDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_constant(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            StructDeclaration(decl_id) => {
                let TyStructDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_struct(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            EnumDeclaration(decl_id) => {
                let TyEnumDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_enum(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            FunctionDeclaration(decl_id) => {
                let TyFunctionDeclaration { visibility, .. } = check!(
                    CompileResult::from(de_get_function(decl_id.clone(), &decl_id.span())),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                visibility
            }
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            VariableDeclaration(decl) => decl.mutability.visibility(),
        };
        ok(visibility, warnings, errors)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyConstantDeclaration {
    pub name: Ident,
    pub value: ty::TyExpression,
    pub(crate) visibility: Visibility,
}

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TyTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TyFunctionParameter>,
    pub return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub return_type_span: Span,
    pub attributes: AttributesMap,
}

impl CopyTypes for TyTraitFn {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type.copy_types(type_mapping);
    }
}

impl TyTraitFn {
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TyFunctionDeclaration {
        TyFunctionDeclaration {
            purity: self.purity,
            name: self.name.clone(),
            body: TyCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            span: self.name.span(),
            attributes: self.attributes.clone(),
            return_type: self.return_type,
            initial_return_type: self.return_type,
            return_type_span: self.return_type_span.clone(),
            visibility: Visibility::Public,
            type_parameters: vec![],
            is_contract_call: mode == Mode::ImplAbiFn,
        }
    }
}

/// Represents the left hand side of a reassignment -- a name to locate it in the
/// namespace, and the type that the name refers to. The type is used for memory layout
/// in asm generation.
#[derive(Clone, Debug, Eq)]
pub struct ReassignmentLhs {
    pub kind: ProjectionKind,
    pub type_id: TypeId,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for ReassignmentLhs {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && look_up_type_id(self.type_id) == look_up_type_id(other.type_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    StructField { name: Ident },
    TupleField { index: usize, index_span: Span },
}

impl Spanned for ProjectionKind {
    fn span(&self) -> Span {
        match self {
            ProjectionKind::StructField { name } => name.span(),
            ProjectionKind::TupleField { index_span, .. } => index_span.clone(),
        }
    }
}

impl ProjectionKind {
    pub(crate) fn pretty_print(&self) -> Cow<str> {
        match self {
            ProjectionKind::StructField { name } => Cow::Borrowed(name.as_str()),
            ProjectionKind::TupleField { index, .. } => Cow::Owned(index.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub lhs_base_name: Ident,
    pub lhs_type: TypeId,
    pub lhs_indices: Vec<ProjectionKind>,
    pub rhs: ty::TyExpression,
}

impl CopyTypes for TyReassignment {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.rhs.copy_types(type_mapping);
        self.lhs_type.copy_types(type_mapping);
    }
}
