//! Type checking for Sway.
pub mod ast_node;
pub(crate) mod cei_pattern_analysis;
pub(crate) mod coins_analysis;
pub mod collection_context;
mod module;
pub mod namespace;
mod node_dependencies;
mod program;
mod type_check_analysis;
pub(crate) mod type_check_context;
mod type_check_finalization;
pub use ast_node::*;
pub use namespace::Namespace;
pub(crate) use type_check_analysis::*;
pub(crate) use type_check_context::TypeCheckContext;
pub(crate) use type_check_finalization::*;
