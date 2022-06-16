//! Type checking for Sway.
pub mod ast_node;
mod context;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
pub(crate) use ast_node::*;
pub use ast_node::{TypedConstantDeclaration, TypedDeclaration, TypedFunctionDeclaration};
pub(crate) use context::Context;
pub use module::{TypedModule, TypedSubmodule};
pub use namespace::Namespace;
pub use program::{TypedProgram, TypedProgramKind};
