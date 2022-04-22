//! Type checking for Sway.
pub mod ast_node;
mod namespace;
mod node_dependencies;
mod syntax_tree;
pub(crate) mod type_check_arguments;
pub(crate) use ast_node::*;
pub use ast_node::{TypedConstantDeclaration, TypedDeclaration, TypedFunctionDeclaration};
pub use namespace::Namespace;
pub use namespace::*;
pub use syntax_tree::TreeType;
pub use syntax_tree::TypedParseTree;
pub use type_check_arguments::*;

const ERROR_RECOVERY_DECLARATION: TypedDeclaration = TypedDeclaration::ErrorRecovery;
