mod create_type_id;
mod r#enum;
mod function;
mod monomorphize;
mod storage;
mod r#struct;
mod variable;
pub(crate) use create_type_id::*;
pub use function::*;
pub(crate) use monomorphize::*;
pub use r#enum::*;
pub use r#struct::*;
pub use storage::*;
pub use variable::*;

use super::{
    copy_types::TypeMapping, impl_trait::Mode, CopyTypes, TypedCodeBlock, TypedExpression,
};
use crate::{
    error::*, parse_tree::*, semantic_analysis::TypeCheckedStorageReassignment, type_engine::*,
    Ident,
};
use derivative::Derivative;
use sway_types::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedDeclaration {
    VariableDeclaration(TypedVariableDeclaration),
    ConstantDeclaration(TypedConstantDeclaration),
    FunctionDeclaration(TypedFunctionDeclaration),
    TraitDeclaration(TypedTraitDeclaration),
    StructDeclaration(TypedStructDeclaration),
    EnumDeclaration(TypedEnumDeclaration),
    Reassignment(TypedReassignment),
    ImplTrait {
        trait_name: CallPath,
        span: Span,
        methods: Vec<TypedFunctionDeclaration>,
        type_implementing_for: TypeInfo,
    },
    AbiDeclaration(TypedAbiDeclaration),
    // If type parameters are defined for a function, they are put in the namespace just for
    // the body of that function.
    GenericTypeForFunctionScope {
        name: Ident,
    },
    ErrorRecovery,
    StorageDeclaration(TypedStorageDeclaration),
    StorageReassignment(TypeCheckedStorageReassignment),
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
            Reassignment(ref mut reassignment) => reassignment.copy_types(type_mapping),
            ImplTrait {
                ref mut methods, ..
            } => {
                methods.iter_mut().for_each(|x| x.copy_types(type_mapping));
            }
            // generics in an ABI is unsupported by design
            AbiDeclaration(..) => (),
            StorageDeclaration(..) => (),
            StorageReassignment(..) => (),
            GenericTypeForFunctionScope { .. } | ErrorRecovery => (),
        }
    }
}

impl TypedDeclaration {
    /// Attempt to retrieve the declaration as an enum declaration.
    ///
    /// Returns `None` if `self` is not an `TypedEnumDeclaration`.
    pub(crate) fn as_enum(&self) -> Option<&TypedEnumDeclaration> {
        match self {
            TypedDeclaration::EnumDeclaration(decl) => Some(decl),
            _ => None,
        }
    }

    /// Attempt to retrieve the declaration as a struct declaration.
    ///
    /// Returns `None` if `self` is not a `TypedStructDeclaration`.
    #[allow(dead_code)]
    pub(crate) fn as_struct(&self) -> Option<&TypedStructDeclaration> {
        match self {
            TypedDeclaration::StructDeclaration(decl) => Some(decl),
            _ => None,
        }
    }

    /// Attempt to retrieve the declaration as a function declaration.
    ///
    /// Returns `None` if `self` is not a `TypedFunctionDeclaration`.
    #[allow(dead_code)]
    pub(crate) fn as_function(&self) -> Option<&TypedFunctionDeclaration> {
        match self {
            TypedDeclaration::FunctionDeclaration(decl) => Some(decl),
            _ => None,
        }
    }

    /// Attempt to retrieve the declaration as a variable declaration.
    ///
    /// Returns `None` if `self` is not a `TypedVariableDeclaration`.
    #[allow(dead_code)]
    pub(crate) fn as_variable(&self) -> Option<&TypedVariableDeclaration> {
        match self {
            TypedDeclaration::VariableDeclaration(decl) => Some(decl),
            _ => None,
        }
    }

    /// Attempt to retrieve the declaration as an Abi declaration.
    ///
    /// Returns `None` if `self` is not a `TypedAbiDeclaration`.
    #[allow(dead_code)]
    pub(crate) fn as_abi(&self) -> Option<&TypedAbiDeclaration> {
        match self {
            TypedDeclaration::AbiDeclaration(decl) => Some(decl),
            _ => None,
        }
    }

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
    pub(crate) fn friendly_name(&self) -> &'static str {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(_) => "variable",
            ConstantDeclaration(_) => "constant",
            FunctionDeclaration(_) => "function",
            TraitDeclaration(_) => "trait",
            StructDeclaration(_) => "struct",
            EnumDeclaration(_) => "enum",
            Reassignment(_) => "reassignment",
            ImplTrait { .. } => "impl trait",
            AbiDeclaration(..) => "abi",
            GenericTypeForFunctionScope { .. } => "generic type parameter",
            ErrorRecovery => "error",
            StorageDeclaration(_) => "contract storage declaration",
            StorageReassignment(_) => "contract storage reassignment",
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
            TypedDeclaration::Reassignment(TypedReassignment { rhs, .. }) => rhs.return_type,
            TypedDeclaration::StorageDeclaration(decl) => insert_type(TypeInfo::Storage {
                fields: decl.fields_as_typed_struct_fields(),
            }),
            TypedDeclaration::GenericTypeForFunctionScope { name } => {
                insert_type(TypeInfo::UnknownGeneric { name: name.clone() })
            }
            decl => {
                return err(
                    vec![],
                    vec![CompileError::NotAType {
                        span: decl.span(),
                        name: decl.pretty_print(),
                        actually_is: decl.friendly_name(),
                    }],
                )
            }
        };
        ok(type_id, vec![], vec![])
    }

    pub(crate) fn span(&self) -> Span {
        use TypedDeclaration::*;
        match self {
            VariableDeclaration(TypedVariableDeclaration { name, .. }) => name.span().clone(),
            ConstantDeclaration(TypedConstantDeclaration { name, .. }) => name.span().clone(),
            FunctionDeclaration(TypedFunctionDeclaration { span, .. }) => span.clone(),
            TraitDeclaration(TypedTraitDeclaration { name, .. }) => name.span().clone(),
            StructDeclaration(TypedStructDeclaration { name, .. }) => name.span().clone(),
            EnumDeclaration(TypedEnumDeclaration { span, .. }) => span.clone(),
            Reassignment(TypedReassignment { lhs, .. }) => lhs
                .iter()
                .fold(lhs[0].span(), |acc, this| Span::join(acc, this.span())),
            AbiDeclaration(TypedAbiDeclaration { span, .. }) => span.clone(),
            ImplTrait { span, .. } => span.clone(),
            StorageDeclaration(decl) => decl.span(),
            StorageReassignment(decl) => decl.span(),
            ErrorRecovery | GenericTypeForFunctionScope { .. } => {
                unreachable!("No span exists for these ast node types")
            }
        }
    }

    pub(crate) fn pretty_print(&self) -> String {
        format!(
            "{} declaration ({})",
            self.friendly_name(),
            match self {
                TypedDeclaration::VariableDeclaration(TypedVariableDeclaration {
                    is_mutable,
                    name,
                    type_ascription,
                    body,
                    ..
                }) => {
                    let mut builder = String::new();
                    match is_mutable {
                        VariableMutability::Mutable => builder.push_str("mut"),
                        VariableMutability::Immutable => {}
                        VariableMutability::ExportedConst => builder.push_str("pub const"),
                    }
                    builder.push_str(name.as_str());
                    builder.push_str(": ");
                    builder.push_str(
                        &crate::type_engine::look_up_type_id(*type_ascription).friendly_type_str(),
                    );
                    builder.push_str(" = ");
                    builder.push_str(&body.pretty_print());
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
                TypedDeclaration::Reassignment(TypedReassignment { lhs, .. }) => lhs
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                _ => String::new(),
            }
        )
    }

    pub(crate) fn visibility(&self) -> Visibility {
        use TypedDeclaration::*;
        match self {
            GenericTypeForFunctionScope { .. }
            | Reassignment(..)
            | ImplTrait { .. }
            | StorageDeclaration { .. }
            | StorageReassignment { .. }
            | AbiDeclaration(..)
            | ErrorRecovery => Visibility::Public,
            VariableDeclaration(TypedVariableDeclaration { is_mutable, .. }) => {
                is_mutable.visibility()
            }
            EnumDeclaration(TypedEnumDeclaration { visibility, .. })
            | ConstantDeclaration(TypedConstantDeclaration { visibility, .. })
            | FunctionDeclaration(TypedFunctionDeclaration { visibility, .. })
            | TraitDeclaration(TypedTraitDeclaration { visibility, .. })
            | StructDeclaration(TypedStructDeclaration { visibility, .. }) => *visibility,
        }
    }
}

/// A `TypedAbiDeclaration` contains the type-checked version of the parse tree's `AbiDeclaration`.
#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedAbiDeclaration {
    /// The name of the abi trait (also known as a "contract trait")
    pub(crate) name: Ident,
    /// The methods a contract is required to implement in order opt in to this interface
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    /// The methods provided to a contract "for free" upon opting in to this interface
    // NOTE: It may be important in the future to include this component
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<FunctionDeclaration>,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) span: Span,
}

impl TypedAbiDeclaration {
    pub(crate) fn as_type(&self) -> TypeId {
        let ty = TypeInfo::ContractCaller {
            abi_name: AbiName::Known(self.name.clone().into()),
            address: None,
        };
        insert_type(ty)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedConstantDeclaration {
    pub(crate) name: Ident,
    pub(crate) value: TypedExpression,
    pub(crate) visibility: Visibility,
}

impl CopyTypes for TypedConstantDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.value.copy_types(type_mapping);
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedTraitDeclaration {
    pub(crate) name: Ident,
    pub(crate) interface_surface: Vec<TypedTraitFn>,
    // NOTE: deriving partialeq and hash on this element may be important in the
    // future, but I am not sure. For now, adding this would 2x the amount of
    // work, so I am just going to exclude it
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) methods: Vec<FunctionDeclaration>,
    pub(crate) supertraits: Vec<Supertrait>,
    pub(crate) visibility: Visibility,
}

impl CopyTypes for TypedTraitDeclaration {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.interface_surface
            .iter_mut()
            .for_each(|x| x.copy_types(type_mapping));
        // we don't have to type check the methods because it hasn't been type checked yet
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(PartialEq, Eq)]
pub struct TypedTraitFn {
    pub(crate) name: Ident,
    pub(crate) purity: Purity,
    pub(crate) parameters: Vec<TypedFunctionParameter>,
    pub(crate) return_type: TypeId,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Eq(bound = ""))]
    pub(crate) return_type_span: Span,
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
            body: TypedCodeBlock {
                contents: vec![],
                whole_block_span: self.name.span().clone(),
            },
            parameters: self.parameters.clone(),
            span: self.name.span().clone(),
            return_type: self.return_type,
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
    pub(crate) name: Ident,
    pub(crate) r#type: TypeId,
}

// NOTE: Hash and PartialEq must uphold the invariant:
// k1 == k2 -> hash(k1) == hash(k2)
// https://doc.rust-lang.org/std/collections/struct.HashMap.html
impl PartialEq for ReassignmentLhs {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && look_up_type_id(self.r#type) == look_up_type_id(other.r#type)
    }
}

impl ReassignmentLhs {
    pub(crate) fn span(&self) -> Span {
        self.name.span().clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedReassignment {
    // either a direct variable, so length of 1, or
    // at series of struct fields/array indices (array syntax)
    pub(crate) lhs: Vec<ReassignmentLhs>,
    pub(crate) rhs: TypedExpression,
}

impl CopyTypes for TypedReassignment {
    fn copy_types(&mut self, type_mapping: &TypeMapping) {
        self.rhs.copy_types(type_mapping);
        self.lhs.iter_mut().for_each(
            |ReassignmentLhs {
                 ref mut r#type,
                 name,
                 ..
             }| {
                r#type.update_type(type_mapping, name.span());
            },
        );
    }
}
