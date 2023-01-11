//! The [DeclarationEngine](declaration_engine::DeclarationEngine) allows the compiler to add a layer of
//! separation between [AST nodes](crate::semantic_analysis::ast_node) and declarations.
//!
//! As an interface, you can think of the [DeclarationEngine](declaration_engine::DeclarationEngine)
//! as a mapping from [DeclarationId](declaration_id::DeclarationId) to
//! [DeclarationWrapper](declaration_wrapper::DeclarationWrapper). When a
//! [DeclarationWrapper](declaration_wrapper::DeclarationWrapper) is inserted into the
//! [DeclarationEngine](declaration_engine::DeclarationEngine), a [DeclarationId](declaration_id::DeclarationId)
//! is generated, which is then used to refer to the declaration.

#[allow(clippy::module_inception)]
pub(crate) mod engine;
pub(crate) mod id;
pub(crate) mod mapping;
pub(crate) mod replace_decl_id;
pub(crate) mod wrapper;

pub use engine::*;
pub use id::*;
pub(crate) use mapping::*;
pub(crate) use replace_decl_id::*;
pub(crate) use wrapper::*;
