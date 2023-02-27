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
pub mod id;
pub(crate) mod mapping;
pub(crate) mod r#ref;
pub(crate) mod replace_decls;
pub(crate) mod wrapper;

use std::collections::BTreeMap;

pub use engine::*;
pub(crate) use id::*;
pub(crate) use mapping::*;
pub use r#ref::*;
pub(crate) use replace_decls::*;
use sway_types::Ident;
pub(crate) use wrapper::*;

use crate::language::ty::{TyTraitInterfaceItem, TyTraitItem};

pub(crate) type InterfaceItemMap = BTreeMap<Ident, TyTraitInterfaceItem>;
pub(crate) type ItemMap = BTreeMap<Ident, TyTraitItem>;
