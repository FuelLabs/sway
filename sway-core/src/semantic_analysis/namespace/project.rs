use crate::Ident;

use super::{Module, ModulePathBuf, Package, PackageId};

use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

/// A representation of the bindings in a project.
///
/// The project is divided into two parts
/// - current_package: The package currently being compiled
/// - dependencies: A collection of packages that have already been compiled, and which the
///   current package depends on (possibly transitively).
///
/// This is equivalent to a Rust project. The current project is equivalent to Rust's "crate".
#[derive(Clone, Debug)]
pub struct Project {
    // The package currently being compiled.
    pub current_package: Package,
    // A map of packages that have already been compiled, and which the current package depends on,
    // possibly transitively.
    //
    // Every package contains a map from `Ident` to `PackageId`. These are the direct dependencies
    // of the package, as specified in Forc.toml for the package. The `PackageId` refers to the name
    // that the dependency gives to itself using the 'name' attribute in the dependency's own
    // Forc.toml. The `Ident` in the map is the name a dependent package gives to the dependency
    // when used in the dependent package's '[dependencies]' section.
    external_packages: im::HashMap<PackageId, Package, BuildHasherDefault<FxHasher>>,
}

impl Project {
    // Find the Package referred to by `name`.
    // If `name` is the name of the current package, then the result is the current Package.
    // If not, then look up the PackageId in the current package, and then look up the corresponding
    // package in `external_packages`.
    pub(crate) fn package_from_ident(&self, name: &Ident) -> Option<&Package> {
        if name == self.current_package.name() {
            Some(&self.current_package)
        } else if let Some(package_id) = self.current_package.external_package_id(name) {
            self.external_packages.get(package_id)
        } else {
            None
        }
    }

    // Find module in the current project. `mod_path` must be a full path
    pub fn module_from_full_path(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(!mod_path.is_empty());
        if let Some(package) = self.package_from_ident(&mod_path[0]) {
            TODO: if package != current_package change mod_path[0] to the package's own name
            package.module_from_full_path(mod_path)
        } else {
            None
        }
    }
}
