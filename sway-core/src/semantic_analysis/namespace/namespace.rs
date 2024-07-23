use crate::{
    build_config::ExperimentalFlags,
    language::{ty, CallPath, Visibility},
    Engines, Ident, TypeId,
};

use super::{
    module::Module,
    root::{ResolvedDeclaration, Root},
    submodule_namespace::SubmoduleNamespace,
    trait_map::ResolvedTraitImplItem,
    contract_helpers::*,
    ModulePath, ModulePathBuf,
};

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::span::Span;

/// Enum used to pass a value asking for insertion of type into trait map when an implementation
/// of the trait cannot be found.
#[derive(Debug)]
pub enum TryInsertingTraitImplOnFailure {
    Yes,
    No,
}

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug)]
pub struct Namespace {
    /// The `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `init` namespace before type-checking begins.
    pub(crate) root: Root,
    /// An absolute path from the `root` that represents the module location.
    ///
    /// The path of the root module in a package is `[package_name]`. If a module `X` is a submodule
    /// of module `Y` which is a submodule of the root module in the package `P`, then the path is
    /// `[P, Y, X]`.
    pub(crate) current_mod_path: ModulePathBuf,
    /// True if the current package is a contract, false otherwise.
    is_contract_package: bool
}

impl Namespace {
    /// Initialize the namespace 
    pub fn new(handler: &Handler, engines: &Engines, package_name: Ident, span: Option<Span>, contract_id: Option<String>, experimental: crate::ExperimentalFlags) -> Result<Self, ErrorEmitted> {
	let root = Root::new(package_name.clone(), span);
	let mut res = Self {
	    root,
	    current_mod_path: vec!(package_name),
	    is_contract_package: contract_id.is_some(),
	};
	if let Some(id) = contract_id {
	    bind_contract_id_in_root_module(handler, engines, id, &mut res, experimental)?;
	}
	Ok(res)
    }

    pub fn next_package(&mut self, handler: &Handler, engines: &Engines, next_package_name: Ident, span: Option<Span>, contract_id: Option<String>, experimental: ExperimentalFlags) -> Result<(), ErrorEmitted> {
	self.root.next_package(next_package_name, span);
	self.current_mod_path = vec!(self.root.current_package_name().clone());
	self.is_contract_package = contract_id.is_some();
	self.import_implicits(&self.current_mod_path.clone());
	if let Some(id) = contract_id {
	    bind_contract_id_in_root_module(handler, engines, id, self, experimental)?;
	}
	Ok(())
    }

    // Import core::prelude (not for core), std::prelude (not for core and std)
    // and contract_id (only for contract packages)
    fn import_implicits(&mut self, _mod_path: &ModulePathBuf) {
	// TODO
	// if next_package_name != "core" && root.external_modules.contains("core") { import core prelude }
	// if next_package_name != "core" && next_package_name != "std" && root.external_modules.contains("std") { import std prelude }
	// if self.is_contract_package { import ::CONTRACT_ID }
    }

    pub fn current_module(&self) -> &Module {
	self.root.module_in_current_package(&self.current_mod_path)
            .unwrap_or_else(|| panic!("Could not retrieve submodule for mod_path."))
    }
    
    pub fn current_module_mut(&mut self) -> &mut Module {
	self.root.module_mut_in_current_package(&self.current_mod_path)
            .unwrap_or_else(|| panic!("Could not retrieve submodule for mod_path."))
    }

    pub(crate) fn current_module_has_submodule(&self, submod_name: &Ident) -> bool {
	self.current_module().submodule(&[submod_name.clone()]).is_some()
    }

    pub fn current_package_name(&self) -> &Ident {
	self.root.current_package_name()
    }
    
//    /// Initialise the namespace at its root from the given initial namespace.
//    /// If the root module contains submodules these are now considered external.
//    pub fn init_root(root: &mut Root) -> Self {
//        assert!(
//            !root.module.is_external,
//            "The root module must not be external during compilation"
//        );
//        let mod_path = vec![];
//
//        // A copy of the root module is used to initialize every new submodule in the program.
//        //
//        // Every submodule that has been added before calling init_root is now considered
//        // external, which we have to enforce at this point.
//        fn set_submodules_external(module: &mut Module) {
//            for (_, submod) in module.submodules_mut().iter_mut() {
//                if !submod.is_external {
//                    submod.is_external = true;
//                    set_submodules_external(submod);
//                }
//            }
//        }
//
//        set_submodules_external(&mut root.module);
//        // The init module itself is not external
//        root.module.is_external = false;
//
//        Self {
//            init: root.module.clone(),
//            root: root.clone(),
//            mod_path,
//        }
//    }

    /// A reference to the path of the module currently being processed.
    pub fn current_mod_path(&self) -> &ModulePathBuf {
        &self.current_mod_path
    }

    /// Prepends the module path into the prefixes.
    pub fn prepend_module_path<'a>(
        &'a self,
        prefixes: impl IntoIterator<Item = &'a Ident>,
    ) -> ModulePathBuf {
        self.current_mod_path.iter().chain(prefixes).cloned().collect()
    }

//    /// A reference to the root of the project namespace.
//    pub fn root(&self) -> &Root {
//        &self.root
//    }

    pub fn current_package_root_module(&self) -> &Module {
        &self.root.current_package_root_module()
    }

    pub fn module_from_absolute_path(
        &self,
        path: &ModulePathBuf,
    ) -> Option<&Module> {
        self.root.module_from_absolute_path(path)
    }

    // Like module_from_absolute_path, but throws an error if the module is not found
    pub fn require_module_from_absolute_path(
        &self,
	handler: &Handler,
        path: &ModulePathBuf,
    ) -> Result<&Module, ErrorEmitted> {
	self.root.require_module_in_current_package(handler, path)
    }

    /// Returns true if the current module being checked is a direct or indirect submodule of
    /// the module given by the `absolute_module_path`.
    ///
    /// The current module being checked is determined by `mod_path`.
    ///
    /// E.g., the `mod_path` `[fist, second, third]` of the root `foo` is a submodule of the module
    /// `[foo, first]`. Note that the `mod_path` does not contain the root name, while the
    /// `absolute_module_path` always contains it.
    ///
    /// If the current module being checked is the same as the module given by the `absolute_module_path`,
    /// the `true_if_same` is returned.
    pub(crate) fn module_is_submodule_of(
        &self,
        absolute_module_path: &ModulePath,
        true_if_same: bool,
    ) -> bool {
        let is_submodule = absolute_module_path
            .iter()
            .zip(self.current_mod_path.iter())
            .all(|(left, right)| left == right);

        if is_submodule {
            if self.current_mod_path.len() == absolute_module_path.len() {
                true_if_same
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Returns true if the module given by the `absolute_module_path` is external
    /// to the current package. External modules are imported in the `Forc.toml` file.
    pub(crate) fn module_is_external(&self, absolute_module_path: &ModulePath) -> bool {
        assert!(!absolute_module_path.is_empty(), "Absolute module path must have at least one element, because it always contains the package name.");

        self.root.current_package_name() != &absolute_module_path[0]
    }

    pub fn get_root_trait_item_for_type(
        &self,
        handler: &Handler,
        engines: &Engines,
        name: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
    ) -> Result<ResolvedTraitImplItem, ErrorEmitted> {
        self.root
            .current_package_root_module()
            .current_items()
            .implemented_traits
            .get_trait_item_for_type(handler, engines, name, type_id, as_trait)
    }

    pub fn resolve_root_symbol(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &ModulePath,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        self.root
            .resolve_symbol(handler, engines, mod_path, symbol, self_type)
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        self.root
            .resolve_symbol(handler, engines, &self.current_mod_path, symbol, self_type)
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol_typed(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        self.resolve_symbol(handler, engines, symbol, self_type)
            .map(|resolved_decl| resolved_decl.expect_typed())
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_typed(
        &self,
        handler: &Handler,
        engines: &Engines,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        self.resolve_call_path(handler, engines, call_path, self_type)
            .map(|resolved_decl| resolved_decl.expect_typed())
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        self.root
            .resolve_call_path(handler, engines, &self.current_mod_path, call_path, self_type)
    }

    pub(crate) fn enter_submodule(&mut self, mod_name: Ident, visibility: Visibility, module_span: Span) -> SubmoduleNamespace {
	if !self.current_module().submodules().contains_key(&mod_name.to_string()) {
	    let submod_path = self.current_module_mut().add_new_submodule(&mod_name, visibility, Some(module_span));
	    self.import_implicits(&submod_path);
	}
	let new_mod = self.current_module().submodules().get(&mod_name.to_string()).unwrap();
	// TODO: Do we need to return a SubmoduleNamespace? Can't we just push the new name onto
	// self.current_mod_path and pop it when done with the submodule? That's what happens in the
	// collection phase (which uses push_new_submodule and pop_submodule).
	let parent_mod_path = self.current_mod_path.clone();
	self.current_mod_path = new_mod.mod_path().to_vec();
	SubmoduleNamespace {
	    namespace: self,
	    parent_mod_path,
	}
    }

    // TODO: Do we need to return a SubmoduleNamespace? Can't we just push the new name onto
    // self.current_mod_path and pop it when done with the submodule? That's what happens in the
    // collection phase (which uses push_new_submodule and pop_submodule).
    // Replace the submodule path with the original module path.
    // This ensures that the namespace's module path is reset when ownership over it is
    // relinquished from the SubmoduleNamespace.
    pub(crate) fn leave_submodule(&mut self, parent_mod_path: &ModulePathBuf) {
	self.current_mod_path = parent_mod_path.clone();
    }

//    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
//    ///
//    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
//    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
//    /// back to the original path so that we can continue type-checking the current module after
//    /// finishing with the dependency.
//    pub(crate) fn enter_submodule(
//        &mut self,
//        engines: &Engines,
//        mod_name: Ident,
//        visibility: Visibility,
//        module_span: Span,
//    ) -> SubmoduleNamespace {
//        let init = self.init.clone();
//        let is_external = self.module(engines).is_external;
//        let submod_path: Vec<_> = self
//            .mod_path
//            .iter()
//            .cloned()
//            .chain(Some(mod_name.clone()))
//            .collect();
//        self.module_mut(engines)
//            .submodules
//            .entry(mod_name.to_string())
//            .or_insert(init.new_submodule_from_init(
//                mod_name,
//                visibility,
//                Some(module_span),
//                is_external,
//                submod_path.clone(),
//            ));
//        let parent_mod_path = std::mem::replace(&mut self.current_mod_path, submod_path.clone());
//        SubmoduleNamespace {
//            namespace: self,
//            parent_mod_path,
//        }
//    }

    /// Pushes a new submodule to the namespace's module hierarchy.
    pub fn push_new_submodule(
        &mut self,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
    ) {
	self.enter_submodule(mod_name, visibility, module_span);
    }
 
    /// Pops the current submodule from the namespace's module hierarchy.
    pub fn pop_submodule(&mut self) {
        self.current_mod_path.pop();
    }

    pub(crate) fn star_import_to_current_module(&mut self, handler: &Handler, engines: &Engines, src: &ModulePath, visibility: Visibility) -> Result<(), ErrorEmitted> {
	self.root.star_import(handler, engines, src, &self.current_mod_path, visibility)
    }

    pub(crate) fn variant_star_import_to_current_module(&mut self, handler: &Handler, engines: &Engines, src: &ModulePath, enum_name: &Ident, visibility: Visibility) -> Result<(), ErrorEmitted> {
	self.root.variant_star_import(handler, engines, src, &self.current_mod_path, enum_name, visibility)
    }
    
    pub(crate) fn self_import_to_current_module(&mut self, handler: &Handler, engines: &Engines, src: &ModulePath, alias: Option<Ident>, visibility: Visibility) -> Result<(), ErrorEmitted> {
	self.root.self_import(handler, engines, src, &self.current_mod_path, alias, visibility)
    }

    pub(crate) fn item_import_to_current_module(&mut self, handler: &Handler, engines: &Engines, src: &ModulePath, item: &Ident, alias: Option<Ident>, visibility: Visibility) -> Result<(), ErrorEmitted> {
	self.root.item_import(handler, engines, src, item, &self.current_mod_path, alias, visibility)
    }

    pub(crate) fn variant_import_to_current_module(&mut self, handler: &Handler, engines: &Engines, src: &ModulePath, enum_name: &Ident, variant_name: &Ident, alias: Option<Ident>, visibility: Visibility) -> Result<(), ErrorEmitted> {
	self.root.variant_import(handler, engines, src, enum_name, variant_name, &self.current_mod_path, alias, visibility)
    }
}
