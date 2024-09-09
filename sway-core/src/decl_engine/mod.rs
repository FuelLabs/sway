//! The [DeclEngine](engine::DeclEngine) allows the compiler to add a layer of
//! separation between [AST nodes](crate::semantic_analysis::ast_node) and
//! declarations.
//!
//! As an interface, you can think of the [DeclEngine](engine::DeclEngine) as a
//! mapping from [DeclId](id::DeclId) to [DeclWrapper](wrapper::DeclWrapper).
//! When a [DeclWrapper](wrapper::DeclWrapper) is inserted into the
//! [DeclEngine](engine::DeclEngine), a [DeclId](id::DeclId) is generated, which
//! is then used to refer to the declaration.

pub mod associated_item_decl_id;
#[allow(clippy::module_inception)]
pub(crate) mod engine;
pub mod id;
pub(crate) mod interface_decl_id;
pub(crate) mod mapping;
pub(crate) mod parsed_engine;
pub mod parsed_id;
pub(crate) mod r#ref;
pub(crate) mod replace_decls;

use std::collections::BTreeMap;

pub(crate) use associated_item_decl_id::*;
pub use engine::*;
pub(crate) use id::*;
pub use interface_decl_id::*;
pub(crate) use mapping::*;
pub use r#ref::*;
pub(crate) use replace_decls::*;
use sway_types::Ident;

use crate::{
    language::ty::{TyTraitInterfaceItem, TyTraitItem},
    TypeId,
};

pub(crate) type InterfaceItemMap = BTreeMap<(Ident, TypeId), TyTraitInterfaceItem>;
pub(crate) type ItemMap = BTreeMap<(Ident, TypeId), TyTraitItem>;
