//! Type checking for Sway.
pub mod ast_node;
pub(crate) mod cei_pattern_analysis;
pub(crate) mod coins_analysis;
mod method_lookup;
mod module;
pub mod namespace;
mod node_dependencies;
pub mod program;
pub mod symbol_collection_context;
pub mod symbol_resolve;
pub mod symbol_resolve_context;
mod type_check_analysis;
pub(crate) mod type_check_context;
mod type_check_finalization;
pub(crate) mod type_resolve;

pub use ast_node::*;
pub use namespace::Namespace;
use sway_macros::generate_visitor;
use sway_types::BaseIdent;
pub(crate) use type_check_analysis::*;
pub use type_check_context::TypeCheckContext;
pub(crate) use type_check_finalization::*;

// Visitor

use crate::{
    AbiName, CallPath, GenericTypeArgument, Ident, Span, TraitConstraint, TypeBinding, TypeId, TypeParameter, ast_elements::type_parameter::{ConstGenericParameter, GenericTypeParameter}, decl_engine::{DeclId, DeclRef, DeclRefFunction}, language::{
        AsmOp, AsmRegister, LazyOp, Literal, Purity, Visibility, ty::{TyCodeBlock, TyDecl, TyFunctionDeclKind, TyFunctionParameter}
    }, transform::Attributes, ty::{
        ContractCallParams, TyAsmRegisterDeclaration, TyConfigurableDecl, TyConstGenericDecl,
        TyConstantDecl, TyEnumDecl, TyEnumVariant, TyExpression, TyFunctionDecl,
        TyIntrinsicFunctionKind, TyReassignment, TyScrutinee, TyStorageAccess, TyStructDecl,
        TyStructExpressionField, TyStructField, VariableMutability,
    }
};
use indexmap::IndexMap;

generate_visitor! {
    const VISIT_GENERIC_TYPE_ARGUMENT_INITIAL_TYPE_ID: bool = true,
    TyFunctionDecl,
    TyFunctionDeclKind,
    GenericTypeArgument,
    Vec<TypeParameter>,
    Option<TypeId>,
    TyCodeBlock,
    Vec<TyFunctionParameter>,
    TypeId,
    Literal,
    CallPath,
    Vec<(Ident, TyExpression)>,
    DeclRefFunction,
    Option<ContractCallParams>,
    Option<TypeBinding<()>>,
    IndexMap<String, TyExpression>,
    Option<Box<TyExpression>>,
    LazyOp,
    Box<TyExpression>,
    Span,
    Box<TyConstantDecl>,
    Box<TyConfigurableDecl>,
    Option<CallPath>,
    Box<TyConstGenericDecl>,
    Ident,
    VariableMutability,
    Vec<TyExpression>,
    DeclId<TyStructDecl>,
    Vec<TyStructExpressionField>,
    TypeBinding<CallPath>,
    Vec<TyScrutinee>,
    Vec<TyAsmRegisterDeclaration>,
    Vec<AsmOp>,
    Option<AsmRegister>,
    TyStructField,
    usize,
    DeclRef<DeclId<TyEnumDecl>>,
    TyStorageAccess,
    TyIntrinsicFunctionKind,
    AbiName,
    TyEnumVariant,
    Box<TyReassignment>,
    Option<(AsmRegister, Span)>,
    TyDecl,
    Option<TyDecl>,
    Attributes,
    Visibility,
    bool,
    Purity,
    Vec<(Ident, Vec<TraitConstraint>)>,
    GenericTypeParameter,
    ConstGenericParameter,
}
