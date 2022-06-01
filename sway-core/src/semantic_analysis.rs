//! Type checking for Sway.
pub mod ast_node;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
pub(crate) mod type_check_arguments;
pub(crate) use ast_node::*;
pub use ast_node::{TypedConstantDeclaration, TypedDeclaration, TypedFunctionDeclaration};
pub use module::{TypedModule, TypedSubmodule};
pub use namespace::Namespace;
pub use program::{TypedProgram, TypedProgramKind};
pub use type_check_arguments::*;

const ERROR_RECOVERY_DECLARATION: TypedDeclaration = TypedDeclaration::ErrorRecovery;
