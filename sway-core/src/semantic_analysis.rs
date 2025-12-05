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
pub(crate) use type_check_analysis::*;
pub use type_check_context::TypeCheckContext;
pub(crate) use type_check_finalization::*;

// Visitor

use crate::{
    TypeBinding, TypeId, ast_elements::binding::EmptyTypeBinding, decl_engine::{DeclId, DeclRefEnum, DeclRefFunction}, language::{
        CallPath, Literal, ty::{
            AbiDecl, ConfigurableDecl, ConstGenericDecl, ConstantDecl, EnumDecl, EnumVariantDecl, FunctionDecl, ImplSelfOrTrait, StorageDecl, StructDecl, TraitDecl, TraitTypeDecl, TyConstGenericDecl, TyStructDecl, TypeAliasDecl
        }
    }, ty::TyConstantDecl
};

generate_visitor! {
    AbiDecl,
    ConfigurableDecl,
    ConstantDecl,
    ConstGenericDecl,
    DeclRefEnum,
    DeclRefFunction,
    DeclId<TyStructDecl>,
    EnumDecl,
    EnumVariantDecl,
    EmptyTypeBinding,
    FunctionDecl,
    ImplSelfOrTrait,
    Literal,
    StorageDecl,
    StructDecl,
    TraitDecl,
    TraitTypeDecl,
    TypeBinding<CallPath>,
    TyConstantDecl,
    TyConstGenericDecl,
    TypeAliasDecl,
    TypeId,
}
