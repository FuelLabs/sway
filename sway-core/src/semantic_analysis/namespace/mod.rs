mod contract_helpers;
mod lexical_scope;
mod module;
#[allow(clippy::module_inception)]
mod namespace;
mod package;
mod resolved_declaration;
mod trait_map;

pub use contract_helpers::*;
pub use lexical_scope::{Items, LexicalScope, LexicalScopeId, LexicalScopePath};
pub use module::module_not_found;
pub use module::Module;
pub use namespace::Namespace;
pub use package::Package;
pub use resolved_declaration::ResolvedDeclaration;
pub(crate) use trait_map::IsExtendingExistingImpl;
pub(crate) use trait_map::IsImplSelf;
pub(super) use trait_map::ResolvedTraitImplItem;
pub use trait_map::TraitMap;
pub use trait_map::TryInsertingTraitImplOnFailure;

use sway_types::Ident;

type ModuleName = String;
pub type ModulePath = [Ident];
pub type ModulePathBuf = Vec<Ident>;
