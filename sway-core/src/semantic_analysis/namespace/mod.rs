mod items;
pub mod module;
#[allow(clippy::module_inception)]
pub mod namespace;
mod root;
mod trait_map;

pub use items::Items;
pub use module::Module;
pub use namespace::Namespace;
pub use root::Root;

use crate::Ident;

pub type Path = [Ident];
pub type PathBuf = Vec<Ident>;
