use super::{module::Module, Ident, ModuleName};
use crate::{
    language::Visibility,
    namespace::ModulePathBuf,
};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use sway_types::{span::Span, ProgramId};

/// The root module, from which all other module dependencies can be accessed.
///
/// This is equivalent to the "crate root" of a Rust crate.
///
/// We use a custom type for the `Root` in order to ensure that methods that only work with
/// canonical paths, or that use canonical paths internally, are *only* called from the root. This
/// normally includes methods that first lookup some canonical path via `use_synonyms` before using
/// that canonical path to look up the symbol declaration.
#[derive(Clone, Debug)]
pub struct Root {
    // The contents of the package being compiled.
    root_module: Module,
    // Program id for the package.
    program_id: ProgramId,
    // True if the current package is a contract, false otherwise.
    is_contract_package: bool,
    // The external dependencies of the current package. Note that an external package is
    // represented as a `Root` object. This is because external packages may have their own external
    // dependencies which are needed for lookups, but which are not directly accessible to the
    // current package.
    pub external_packages: im::HashMap<ModuleName, Root, BuildHasherDefault<FxHasher>>,
}

impl Root {
    // Create a new root object with a root module in the current package.
    //
    // To ensure the correct initialization the factory functions `package_root_without_contract_id`
    // and `package_root_with_contract_id` are supplied in `contract_helpers`.
    //
    // External packages must be added afterwards by calling `add_external`
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
    // may be referred to by a different name in the forc.toml file than the actual name of the
    // package.
    pub fn add_external(&mut self, package_name: String, external_package: Root) {
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

    pub fn package_name(&self) -> &Ident {
        self.root_module.name()
    }

    pub fn program_id(&self) -> ProgramId {
        self.program_id
    }

    pub(crate) fn check_path_is_in_package(&self, mod_path: &ModulePathBuf) -> bool {
        !mod_path.is_empty() && mod_path[0] == *self.root_module.name()
    }

    pub(crate) fn package_relative_path(mod_path: &ModulePathBuf) -> ModulePathBuf {
        mod_path[1..].to_vec()
    }

    pub(super) fn is_contract_package(&self) -> bool {
        self.is_contract_package
    }

    // Find module in the current environment. `mod_path` must be a fully qualified path
    pub fn module_from_absolute_path(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if mod_path[0] == *self.root_module.name() {
            self.root_module.submodule(&package_relative_path)
        } else if let Some(external_package) = self.external_packages.get(&mod_path[0].to_string())
        {
            external_package
                .root_module()
                .submodule(&package_relative_path)
        } else {
            None
        }
    }
}
