//! The [DeclEngine](engine::DeclEngine) allows the compiler to add a layer of
//! separation between [AST nodes](crate::semantic_analysis::ast_node) and
//! declarations.
//!
//! As an interface, you can think of the [DeclEngine](engine::DeclEngine) as a
//! mapping from [DeclId](id::DeclId) to [DeclWrapper](wrapper::DeclWrapper).
//! When a [DeclWrapper](wrapper::DeclWrapper) is inserted into the
//! [DeclEngine](engine::DeclEngine), a [DeclId](id::DeclId) is generated, which
//! is then used to refer to the declaration.

#[allow(clippy::module_inception)]
pub(crate) mod engine;
pub(crate) mod mapping;
pub(crate) mod r#ref;
pub(crate) mod replace_decl_id;
pub(crate) mod wrapper;

pub use engine::*;
pub(crate) use mapping::*;
pub use r#ref::*;
pub(crate) use replace_decl_id::*;
pub(crate) use wrapper::*;
