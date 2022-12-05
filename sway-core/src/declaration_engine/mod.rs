//! The [DeclarationEngine](declaration_engine::DeclarationEngine) allows the compiler to add a layer of
//! separation between [AST nodes](crate::semantic_analysis::ast_node) and declarations.
//!
//! As an interface, you can think of the [DeclarationEngine](declaration_engine::DeclarationEngine)
//! as a mapping from [DeclarationId](declaration_id::DeclarationId) to
//! [DeclarationWrapper](declaration_wrapper::DeclarationWrapper). When a
//! [DeclarationWrapper](declaration_wrapper::DeclarationWrapper) is inserted into the
//! [DeclarationEngine](declaration_engine::DeclarationEngine), a [DeclarationId](declaration_id::DeclarationId)
//! is generated, which is then used to refer to the declaration.

pub(crate) mod decl_mapping;
#[allow(clippy::module_inception)]
pub(crate) mod declaration_engine;
pub(crate) mod declaration_id;
pub(crate) mod declaration_wrapper;
pub(crate) mod replace_declaration_id;

pub(crate) use decl_mapping::*;
pub use declaration_engine::*;
pub(crate) use declaration_id::*;
pub(crate) use replace_declaration_id::*;
