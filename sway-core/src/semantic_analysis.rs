//! Type checking for Sway.
pub mod ast_node;
pub(crate) mod cei_pattern_analysis;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
pub(crate) mod storage_only_types;
mod type_check_context;
pub use ast_node::*;
pub use namespace::Namespace;
pub(crate) use type_check_context::TypeCheckContext;
