mod lexical_scope;
mod module;
#[allow(clippy::module_inception)]
mod namespace;
mod root;
mod submodule_namespace;
mod trait_map;

pub use lexical_scope::{Items, LexicalScope, LexicalScopeId, LexicalScopePath};
pub use module::Module;
pub use namespace::Namespace;
pub use namespace::TryInsertingTraitImplOnFailure;
pub use root::Root;
pub(super) use trait_map::IsExtendingExistingImpl;
pub(super) use trait_map::IsImplSelf;
pub(super) use trait_map::TraitMap;

use sway_types::Ident;

type ModuleName = String;

pub type Path = [Ident];
pub type PathBuf = Vec<Ident>;
