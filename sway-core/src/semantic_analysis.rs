//! Type checking for Sway.
pub mod ast_node;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
mod type_check_context;
pub(crate) use ast_node::*;
pub use ast_node::{TypedConstantDeclaration, TypedDeclaration, TypedFunctionDeclaration};
pub use module::{TypedModule, TypedSubmodule};
pub use namespace::Namespace;
pub use program::{TypedProgram, TypedProgramKind};
pub(crate) use type_check_context::TypeCheckContext;
