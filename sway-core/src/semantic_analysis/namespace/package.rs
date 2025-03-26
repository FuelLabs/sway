use super::{module::Module, Ident, ModuleName, PackageId};
use crate::{language::Visibility, namespace::ModulePathBuf};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use sway_types::{span::Span, ProgramId};

/// A representation of the bindings in a package. The package's module structure can be accessed
/// via the root module.
///
/// This is equivalent to a Rust crate. The root module is equivalent to Rust's "crate root".
#[derive(Clone, Debug)]
pub struct Package {
    // The contents of the package being compiled.
    root_module: Module,
    // Program id for the package.
    program_id: ProgramId,
    // True if the current package is a contract, false otherwise.
    is_contract_package: bool,
    // The external dependencies of the current package as specified in the package's Forc.toml.
    //
    // If the dependency is specified as
    //
    // name_of_dependency = { location_of_dependency }
    //
    // then the map key for that dependency will be name_of_dependency, and the map value will be a
    // unique identifier that the package manager assigns to the package it finds at
    // location_of_dependency.
    external_packages: im::HashMap<Ident, PackageId, BuildHasherDefault<FxHasher>>,
}

impl Package {
    // Create a new `Package` object with a root module.
    //
    // To ensure the correct initialization the factory function `package_with_contract_id` is
    // supplied in `contract_helpers`.
    //
    // External packages must be added afterwards by calling `add_external`.
    pub fn new(
        package_name: Ident,
        span: Option<Span>,
        program_id: ProgramId,
        is_contract_package: bool,
    ) -> Self {
        // The root module must be public
        let module = Module::new(package_name, Visibility::Public, span, &vec![]);
        Self {
            root_module: module,
            program_id,
            is_contract_package,
            external_packages: Default::default(),
        }
    }

    // Add an external package to this package.
    //
    // If the Forc.toml file of the current `Package` contains the lines
    //
    // [dependencies]
    // foo = { bar }
    //
    // then `package_name` should be "foo", and package_id should be the id of the package referred
    // to by "bar"
    pub fn add_external(&mut self, package_name: Ident, package_id: PackageId) {
        // This should be ensured by the package manager
        assert!(!self.external_packages.contains_key(&package_name));
        self.external_packages.insert(package_name, package_id);
    }

    pub fn external_package_id(&self, package_name: &Ident) -> Option<&PackageId> {
        self.external_packages.get(package_name)
    }

    pub fn root_module(&self) -> &Module {
        &self.root_module
    }

    pub fn root_module_mut(&mut self) -> &mut Module {
        &mut self.root_module
    }

    pub fn name(&self) -> &Ident {
        self.root_module.name()
    }

    pub fn program_id(&self) -> ProgramId {
        self.program_id
    }

    pub(crate) fn is_path_in_package(&self, mod_path: &ModulePathBuf) -> bool {
        !mod_path.is_empty() && mod_path[0] == *self.root_module.name()
    }

    pub(crate) fn package_relative_path(mod_path: &ModulePathBuf) -> ModulePathBuf {
        mod_path[1..].to_vec()
    }

    pub(super) fn is_contract_package(&self) -> bool {
        self.is_contract_package
    }

    // Find a module in the current package. `mod_path` must be a full path, and the first
    // identifier in the path must be the current package name
    pub fn module_from_full_path(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(self.is_path_in_package(mod_path));
        let package_relative_path = Self::package_relative_path(mod_path);
        self.root_module.submodule(&package_relative_path)
    }
}
