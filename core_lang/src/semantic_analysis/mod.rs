pub mod ast_node;
mod namespace;
mod node_dependencies;
mod syntax_tree;
pub(crate) use ast_node::{TypedAstNode, TypedAstNodeContent, TypedExpression};
pub use ast_node::{TypedDeclaration, TypedFunctionDeclaration};
pub use namespace::Namespace;
pub(crate) use syntax_tree::{TreeType, TypedParseTree};

const ERROR_RECOVERY_DECLARATION: TypedDeclaration = TypedDeclaration::ErrorRecovery;
