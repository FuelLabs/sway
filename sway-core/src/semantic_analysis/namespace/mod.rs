#[allow(clippy::module_inception)]
mod items;
mod module;
mod namespace;
mod root;
mod submodule_namespace;
mod trait_map;

pub use items::Items;
pub use module::Module;
pub use namespace::Namespace;
pub use root::Root;

use sway_types::Ident;

type ModuleName = String;

pub type Path = [Ident];
pub type PathBuf = Vec<Ident>;
