use std::fmt;

use super::{module::Module, Ident, ModuleName};
use crate::{
    decl_engine::DeclEngine,
    engine_threading::*,
    language::{
        parsed::*,
        ty::{self, StructDecl, TyDecl},
        Visibility,
    },
    namespace::ModulePathBuf,
    TypeId,
};
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{span::Span, ProgramId};

#[derive(Clone, Debug)]
pub enum ResolvedDeclaration {
    Parsed(Declaration),
    Typed(ty::TyDecl),
}

impl DisplayWithEngines for ResolvedDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            ResolvedDeclaration::Parsed(decl) => DisplayWithEngines::fmt(decl, f, engines),
            ResolvedDeclaration::Typed(decl) => DisplayWithEngines::fmt(decl, f, engines),
        }
    }
}

impl DebugWithEngines for ResolvedDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            ResolvedDeclaration::Parsed(decl) => DebugWithEngines::fmt(decl, f, engines),
            ResolvedDeclaration::Typed(decl) => DebugWithEngines::fmt(decl, f, engines),
        }
    }
}

impl PartialEqWithEngines for ResolvedDeclaration {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (ResolvedDeclaration::Parsed(lhs), ResolvedDeclaration::Parsed(rhs)) => {
                lhs.eq(rhs, ctx)
            }
            (ResolvedDeclaration::Typed(lhs), ResolvedDeclaration::Typed(rhs)) => lhs.eq(rhs, ctx),
            // TODO: Right now we consider differently represented resolved declarations to not be
            // equal. This is only used for comparing paths when doing imports, and we will be able
            // to safely remove it once we introduce normalized paths.
            (ResolvedDeclaration::Parsed(_lhs), ResolvedDeclaration::Typed(_rhs)) => false,
            (ResolvedDeclaration::Typed(_lhs), ResolvedDeclaration::Parsed(_rhs)) => false,
        }
    }
}

impl ResolvedDeclaration {
    pub fn is_typed(&self) -> bool {
        match self {
            ResolvedDeclaration::Parsed(_) => false,
            ResolvedDeclaration::Typed(_) => true,
        }
    }

    pub fn resolve_parsed(self, decl_engine: &DeclEngine) -> Declaration {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl,
            ResolvedDeclaration::Typed(ty_decl) => ty_decl
                .get_parsed_decl(decl_engine)
                .expect("expecting valid parsed declaration"),
        }
    }

    pub fn expect_parsed(self) -> Declaration {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl,
            ResolvedDeclaration::Typed(_ty_decl) => panic!(),
        }
    }

    pub fn expect_typed(self) -> ty::TyDecl {
        match self {
            ResolvedDeclaration::Parsed(_) => panic!(),
            ResolvedDeclaration::Typed(ty_decl) => ty_decl,
        }
    }

    pub fn expect_typed_ref(&self) -> &ty::TyDecl {
        match self {
            ResolvedDeclaration::Parsed(_) => panic!(),
            ResolvedDeclaration::Typed(ty_decl) => ty_decl,
        }
    }

    pub(crate) fn to_struct_decl(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl
                .to_struct_decl(handler, engines)
                .map(|id| ResolvedDeclaration::Parsed(Declaration::StructDeclaration(id))),
            ResolvedDeclaration::Typed(decl) => decl.to_struct_decl(handler, engines).map(|id| {
                ResolvedDeclaration::Typed(TyDecl::StructDecl(StructDecl { decl_id: id }))
            }),
        }
    }

    pub(crate) fn visibility(&self, engines: &Engines) -> Visibility {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl.visibility(engines.pe()),
            ResolvedDeclaration::Typed(decl) => decl.visibility(engines.de()),
        }
    }

    pub(crate) fn span(&self, engines: &Engines) -> sway_types::Span {
        match self {
            ResolvedDeclaration::Parsed(decl) => decl.span(engines),
            ResolvedDeclaration::Typed(decl) => decl.span(engines),
        }
    }

    pub(crate) fn return_type(
        &self,
        handler: &Handler,
        engines: &Engines,
    ) -> Result<TypeId, ErrorEmitted> {
        match self {
            ResolvedDeclaration::Parsed(_decl) => unreachable!(),
            ResolvedDeclaration::Typed(decl) => decl.return_type(handler, engines),
        }
    }

    pub(crate) fn is_trait(&self) -> bool {
        match self {
            ResolvedDeclaration::Parsed(decl) => {
                matches!(decl, Declaration::TraitDeclaration(_))
            }
            ResolvedDeclaration::Typed(decl) => {
                matches!(decl, TyDecl::TraitDecl(_))
            }
        }
    }
}

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
    current_package: Module,
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
            current_package: module,
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
        &self.current_package
    }

    pub fn root_module_mut(&mut self) -> &mut Module {
        &mut self.current_package
    }

    pub fn current_package_name(&self) -> &Ident {
        self.current_package.name()
    }

    pub fn program_id(&self) -> ProgramId {
        self.program_id
    }

    pub(crate) fn check_path_is_in_current_package(&self, mod_path: &ModulePathBuf) -> bool {
        !mod_path.is_empty() && mod_path[0] == *self.current_package.name()
    }

    fn package_relative_path(mod_path: &ModulePathBuf) -> ModulePathBuf {
        mod_path[1..].to_vec()
    }

    pub(super) fn is_contract_package(&self) -> bool {
        self.is_contract_package
    }

    // Find module in the current environment. `mod_path` must be a fully qualified path
    pub fn module_from_absolute_path(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if mod_path[0] == *self.current_package.name() {
            self.current_package.submodule(&package_relative_path)
        } else if let Some(external_package) = self.external_packages.get(&mod_path[0].to_string())
        {
            external_package
                .root_module()
                .submodule(&package_relative_path)
        } else {
            None
        }
    }

    // Find a module in the current package. `mod_path` must be a fully qualified path
    pub(super) fn module_in_current_package(&self, mod_path: &ModulePathBuf) -> Option<&Module> {
        assert!(self.check_path_is_in_current_package(mod_path));
        self.module_from_absolute_path(mod_path)
    }

    // Find mutable module in the current environment. `mod_path` must be a fully qualified path
    pub(super) fn module_mut_from_absolute_path(
        &mut self,
        mod_path: &ModulePathBuf,
    ) -> Option<&mut Module> {
        assert!(!mod_path.is_empty());
        let package_relative_path = Self::package_relative_path(mod_path);
        if *self.current_package.name() == mod_path[0] {
            self.current_package.submodule_mut(&package_relative_path)
        } else if let Some(external_package) =
            self.external_packages.get_mut(&mod_path[0].to_string())
        {
            external_package.module_mut_in_current_package(&package_relative_path)
        } else {
            None
        }
    }

    // Find a mutable module in the current package. `mod_path` must be a fully qualified path
    pub(super) fn module_mut_in_current_package(
        &mut self,
        mod_path: &ModulePathBuf,
    ) -> Option<&mut Module> {
        assert!(self.check_path_is_in_current_package(mod_path));
        self.module_mut_from_absolute_path(mod_path)
    }
}
