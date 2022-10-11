//! Type checking for Sway.
pub mod ast_node;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
mod storage_only_types;
mod type_check_context;

pub use ast_node::TyConstantDeclaration;
pub(crate) use ast_node::*;
pub use module::{TyModule, TySubmodule};
pub use namespace::Namespace;
pub use program::{TyProgram, TyProgramKind};
pub(crate) use type_check_context::TypeCheckContext;
