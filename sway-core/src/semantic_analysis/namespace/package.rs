use super::{module::Module, Ident, ModuleName};
use crate::language::Visibility;
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
    pub program_id: ProgramId,
    // True if the current package is a contract, false otherwise.
    is_contract_package: bool,
    // The external dependencies of the current package. Note that an external package is
    // represented as a `Package` object. This is because external packages may have their own external
    // dependencies which are needed for lookups, but which are not directly accessible to the
    // current package.
    pub external_packages: im::HashMap<ModuleName, Package, BuildHasherDefault<FxHasher>>,
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

    // Add an external package to this package. The package name must be supplied, since the package
    // may be referred to by a different name in the Forc.toml file than the actual name of the
    // package.
    pub fn add_external(&mut self, package_name: String, external_package: Package) {
        // This should be ensured by the package manager
        assert!(!self.external_packages.contains_key(&package_name));
        self.external_packages
            .insert(package_name, external_package);
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

    pub(crate) fn check_path_is_in_package(&self, mod_path: &[Ident]) -> bool {
        !mod_path.is_empty() && mod_path[0] == *self.root_module.name()
    }

    pub(crate) fn package_relative_path(mod_path: &[Ident]) -> &[Ident] {
        &mod_path[1..]
    }

    pub(super) fn is_contract_package(&self) -> bool {
        self.is_contract_package
    }

    // Find module in the current environment. `mod_path` must be a fully qualified path
    pub fn module_from_absolute_path(&self, mod_path: &[Ident]) -> Option<&Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if mod_path[0] == *self.root_module.name() {
            self.root_module.submodule(&package_relative_path)
        } else if let Some(external_package) = self.external_packages.get(mod_path[0].as_str())
        {
            external_package
                .root_module()
                .submodule(&package_relative_path)
        } else {
            None
        }
    }
}
