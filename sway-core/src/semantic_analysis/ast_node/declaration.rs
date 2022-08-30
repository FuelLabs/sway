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

use crate::{error::*, parse_tree::*, semantic_analysis::*, type_system::*};
use derivative::Derivative;
use std::{borrow::Cow, fmt};
use sway_types::{Ident, Span, Spanned};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedDeclaration {
    VariableDeclaration(TypedVariableDeclaration),
    ConstantDeclaration(TypedConstantDeclaration),
    FunctionDeclaration(TypedFunctionDeclaration),
    TraitDeclaration(TypedTraitDeclaration),
    StructDeclaration(TypedStructDeclaration),
    EnumDeclaration(TypedEnumDeclaration),
    ImplTrait(TypedImplTrait),
    AbiDeclaration(TypedAbiDeclaration),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope { name: Ident, type_id: TypeId },
    ErrorRecovery,
    StorageDeclaration(TypedStorageDeclaration),
}

impl CopyTypes for TypedDeclaration {
    /// The entry point to monomorphizing typed declarations. Instantiates all new type ids,
    /// assuming `self` has already been copied.
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(ref mut var_decl) => var_decl.copy_types(type_mapping),
            ConstantDeclaration(ref mut const_decl) => const_decl.copy_types(type_mapping),
            FunctionDeclaration(ref mut fn_decl) => fn_decl.copy_types(type_mapping),
            TraitDeclaration(ref mut trait_decl) => trait_decl.copy_types(type_mapping),
            StructDeclaration(ref mut struct_decl) => struct_decl.copy_types(type_mapping),
            EnumDeclaration(ref mut enum_decl) => enum_decl.copy_types(type_mapping),
            ImplTrait(impl_trait) => impl_trait.copy_types(type_mapping),
            // generics in an ABI is unsupported by design
            AbiDeclaration(..)
            | StorageDeclaration(..)
            | GenericTypeForFunctionScope { .. }
            | ErrorRecovery => (),
        }
    }
}

impl Spanned for TypedDeclaration {
    fn span(&self) -> Span {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait(TypedImplTrait { span, .. }) => span.clone(),
            StorageDeclaration(decl) => decl.span(),
            ErrorRecovery | GenericTypeForFunctionScope { .. } => {
                unreachable!("No span exists for these ast node types")
            }
        }
    }
}

impl fmt::Display for TypedDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    mutability,
                    name,
                    type_ascription,
                    body,
                    ..
                }) => {
                    let mut builder = String::new();
                    match mutability {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::RefMutable => builder.push_str("ref mut"),
                        VariableMutability::Immutable => {}
                        VariableMutability::ExportedConst => builder.push_str("pub const"),
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &crate::type_system::look_up_type_id(*type_ascription).to_string(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&body.to_string());
                    builder
                }
                TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
                    name, ..
                }) => {
                    name.as_str().into()
                }
                TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::StructDeclaration(TypedStructDeclaration { name, .. }) =>
                    name.as_str().into(),
                TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { name, .. }) =>
                    name.as_str().into(),
                _ => String::new(),
            }
        )
    }
}

impl UnresolvedTypeCheck for TypedDeclaration {
    // this is only run on entry nodes, which must have all well-formed types
    fn check_for_unresolved_types(&self) -> Vec<CompileError> {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(decl) => {
                let mut body = decl.body.check_for_unresolved_types();
                body.append(&mut decl.type_ascription.check_for_unresolved_types());
                body
            }
            FunctionDeclaration(decl) => {
                let mut body: Vec<CompileError> = decl
                    .body
                    .contents
                    .iter()
                    .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                    .collect();
                body.append(&mut decl.return_type.check_for_unresolved_types());
                body.append(
                    &mut decl
                        .type_parameters
                        .iter()
                        .map(|x| &x.type_id)
                        .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                        .collect(),
                );
                body.append(
                    &mut decl
                        .parameters
                        .iter()
                        .map(|x| &x.type_id)
                        .flat_map(UnresolvedTypeCheck::check_for_unresolved_types)
                        .collect(),
                );
                body
            }
            ConstantDeclaration(TypedConstantDeclaration { value, .. }) => {
                value.check_for_unresolved_types()
            }
            ErrorRecovery
            | StorageDeclaration(_)
            | TraitDeclaration(_)
            | StructDeclaration(_)
            | EnumDeclaration(_)
            | ImplTrait { .. }
            | AbiDeclaration(_)
            | GenericTypeForFunctionScope { .. } => vec![],
        }
    }
}

impl TypedDeclaration {
    /// Retrieves the declaration as an enum declaration.
    ///
    /// Returns an error if `self` is not a `TypedEnumDeclaration`.
    pub(crate) fn expect_enum(&self) -> CompileResult<&TypedEnumDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypedDeclaration::EnumDeclaration(decl) => ok(decl, warnings, errors),
            decl => {
                errors.push(CompileError::DeclIsNotAnEnum {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// Retrieves the declaration as a struct declaration.
    ///
    /// Returns an error if `self` is not a `TypedStructDeclaration`.
    pub(crate) fn expect_struct(&self) -> CompileResult<&TypedStructDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypedDeclaration::StructDeclaration(decl) => ok(decl, warnings, errors),
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
    /// Returns an error if `self` is not a `TypedFunctionDeclaration`.
    pub(crate) fn expect_function(&self) -> CompileResult<&TypedFunctionDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypedDeclaration::FunctionDeclaration(decl) => ok(decl, warnings, errors),
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
    /// Returns an error if `self` is not a `TypedVariableDeclaration`.
    pub(crate) fn expect_variable(&self) -> CompileResult<&TypedVariableDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypedDeclaration::VariableDeclaration(decl) => ok(decl, warnings, errors),
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
    /// Returns an error if `self` is not a `TypedAbiDeclaration`.
    pub(crate) fn expect_abi(&self) -> CompileResult<&TypedAbiDeclaration> {
        let warnings = vec![];
        let mut errors = vec![];
        match self {
            TypedDeclaration::AbiDeclaration(decl) => ok(decl, warnings, errors),
            decl => {
                errors.push(CompileError::DeclIsNotAnAbi {
                    actually: decl.friendly_name().to_string(),
                    span: decl.span(),
                });
                err(warnings, errors)
            }
        }
    }

    /// friendly name string used for error reporting.
    pub fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
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

    pub(crate) fn return_type(&self) -> CompileResult<TypeId> {
        let type_id = match self {
            TypedDeclaration::VariableDeclaration(TypedVariableDeclaration { body, .. }) => {
                body.return_type
            }
            TypedDeclaration::FunctionDeclaration { .. } => {
                return err(
                    vec![],
                    vec![CompileError::Unimplemented(
                        "Function pointers have not yet been implemented.",
                        self.span(),
                    )],
                )
            }
            TypedDeclaration::StructDeclaration(decl) => decl.create_type_id(),
            TypedDeclaration::EnumDeclaration(decl) => decl.create_type_id(),
            TypedDeclaration::StorageDeclaration(decl) => insert_type(TypeInfo::Storage {
                fields: decl.fields_as_typed_struct_fields(),
            }),
            TypedDeclaration::GenericTypeForFunctionScope { name, type_id } => {
                insert_type(TypeInfo::Ref(*type_id, name.span()))
            }
            decl => {
                return err(
                    vec![],
                    vec![CompileError::NotAType {
                        span: decl.span(),
                        name: decl.to_string(),
                        actually_is: decl.friendly_name(),
                    }],
                )
            }
        };
        ok(type_id, vec![], vec![])
    }

    pub(crate) fn visibility(&self) -> Visibility {
        use TypedDeclaration::*;
        match self {
            GenericTypeForFunctionScope { .. }
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            VariableDeclaration(TypedVariableDeclaration {
                mutability: is_mutable,
                ..
            }) => is_mutable.visibility(),
            EnumDeclaration(TypedEnumDeclaration { visibility, .. })
            | ConstantDeclaration(TypedConstantDeclaration { visibility, .. })
            | FunctionDeclaration(TypedFunctionDeclaration { visibility, .. })
            | TraitDeclaration(TypedTraitDeclaration { visibility, .. })
            | StructDeclaration(TypedStructDeclaration { visibility, .. }) => *visibility,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedConstantDeclaration {
    pub name: Ident,
    pub value: TypedExpression,
    pub(crate) visibility: Visibility,
}

impl CopyTypes for TypedConstantDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedTraitFn {
    pub name: Ident,
    pub(crate) purity: Purity,
    pub parameters: Vec<TypedFunctionParameter>,
    pub return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub return_type_span: Span,
}

impl CopyTypes for TypedTraitFn {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.return_type
            .update_type(type_mapping, &self.return_type_span);
    }
}

impl TypedTraitFn {
    /// This function is used in trait declarations to insert "placeholder" functions
    /// in the methods. This allows the methods to use functions declared in the
    /// interface surface.
    pub(crate) fn to_dummy_func(&self, mode: Mode) -> TypedFunctionDeclaration {
        TypedFunctionDeclaration {
            purity: self.purity,
            name: self.name.clone(),
            body: TypedCodeBlock { contents: vec![] },
            parameters: self.parameters.clone(),
            span: self.name.span(),
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
pub struct TypedReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub lhs_base_name: Ident,
    pub lhs_type: TypeId,
    pub lhs_indices: Vec<ProjectionKind>,
    pub rhs: TypedExpression,
}

impl CopyTypes for TypedReassignment {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.rhs.copy_types(type_mapping);
        self.lhs_type
            .update_type(type_mapping, &self.lhs_base_name.span());
    }
}
