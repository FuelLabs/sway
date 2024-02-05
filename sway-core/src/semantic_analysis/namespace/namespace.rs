use crate::{
    language::{ty, ty::TyTraitItem, CallPath, Visibility},
    Engines, Ident, TypeId,
};

use super::{module::Module, root::Root, submodule_namespace::SubmoduleNamespace, Path, PathBuf};

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
    /// An immutable namespace that consists of the names that should always be present, no matter
    /// what module or scope we are currently checking.
    ///
    /// These include external library dependencies and (when it's added) the `std` prelude.
    ///
    /// This is passed through type-checking in order to initialise the namespace of each submodule
    /// within the project.
    init: Module,
    /// The `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `init` namespace before type-checking begins.
    root: Root,
    /// An absolute path from the `root` that represents the current module being checked.
    ///
    /// E.g. when type-checking the root module, this is equal to `[]`. When type-checking a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub(crate) mod_path: PathBuf,
}

impl Namespace {
    /// Initialise the namespace at its root from the given initial namespace.
    pub fn init_root(init: Module) -> Self {
        let root = Root::from(init.clone());
        let mod_path = vec![];
        Self {
            init,
            root,
            mod_path,
        }
    }

    /// A reference to the path of the module currently being type-checked.
    pub fn mod_path(&self) -> &Path {
        &self.mod_path
    }

    /// Find the module that these prefixes point to
    pub fn find_module_path<'a>(
        &'a self,
        prefixes: impl IntoIterator<Item = &'a Ident>,
    ) -> PathBuf {
        self.mod_path.iter().chain(prefixes).cloned().collect()
    }

    /// A reference to the root of the project namespace.
    pub fn root(&self) -> &Root {
        &self.root
    }

    pub fn root_module(&self) -> &Module {
	&self.root.module
    }

    /// The name of the root module
    pub fn root_module_name(&self) -> &Ident {
	&self.root.module.name
    }
    
    pub fn check_absolute_path_to_submodule(
	&self,
        handler: &Handler,
        path: &[Ident],
    ) -> Result<&Module, ErrorEmitted> {
	self.root.module.check_submodule(handler, path)
    }
    
    pub fn get_root_trait_item_for_type(
	&self,
        handler: &Handler,
        engines: &Engines,
        name: &Ident,
        type_id: TypeId,
        as_trait: Option<CallPath>,
    ) -> Result<TyTraitItem, ErrorEmitted> {
	self.root.module.items().implemented_traits.get_trait_item_for_type(handler, engines, name, type_id, as_trait)
    }

    pub fn resolve_root_symbol(
	&self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
	self.root.resolve_symbol(handler, engines, mod_path, symbol, self_type)
    }
    
    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module(&self) -> &Module {
        &self.root.module[&self.mod_path]
    }

    /// Mutable access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module_mut(&mut self) -> &mut Module {
        &mut self.root.module[&self.mod_path]
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        self.root
            .resolve_symbol(handler, engines, &self.mod_path, symbol, self_type)
    }

    /// Short-hand for calling [Self::resolve_call_path_and_mod_path] with the namespace's `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let (decl, _) =
            self.resolve_call_path_and_mod_path(handler, engines, &self.mod_path, call_path, self_type)?;
        Ok(decl)
    }

    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This is short-hand for concatenating the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    pub(crate) fn resolve_call_path_and_mod_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        mod_path: &Path,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<(ty::TyDecl, Vec<Ident>), ErrorEmitted> {
        let symbol_path: Vec<_> = mod_path
            .iter()
            .chain(&call_path.prefixes)
            .cloned()
            .collect();
        self.root.resolve_symbol_and_mod_path(
            handler,
            engines,
            &symbol_path,
            &call_path.suffix,
            self_type,
        )
    }

    /// Short-hand for calling [Root::resolve_call_path_and_root_type_id] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_and_root_type_id(
        &self,
        handler: &Handler,
        engines: &Engines,
        root_type_id: TypeId,
        as_trait: Option<CallPath>,
        call_path: &CallPath,
        self_type: Option<TypeId>,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
	self.root.resolve_call_path_and_root_type_id(handler, engines, root_type_id, as_trait, call_path, self_type)
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(
        &mut self,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
    ) -> SubmoduleNamespace {
        let init = self.init.clone();
        self.module_mut().submodules.entry(mod_name.to_string()).or_insert(init);
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(mod_name.clone()))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
	// self.module() now refers to a different module, so refetch
	let new_module = self.module_mut();
        new_module.name = mod_name;
        new_module.span = Some(module_span);
        new_module.visibility = visibility;
        new_module.is_external = false;
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }

    /// Import into this namespace a path that contains an asterisk.
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
	self.root.module.star_import(
            handler,
            engines,
            src,
            &self.mod_path,
            is_absolute,
	)
    }

    /// Import into this namespace all variants from the enum `enum_name` from the given `src` module.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        enum_name: &Ident,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.module.variant_star_import(
            handler,
            engines,
            src,
            &self.mod_path,
            enum_name,
            is_absolute,
        )
    }

    /// Import into this namespace a single `item` from a `src` module.
    /// 
    /// The item we want to import is the last item in path because this is a `self` import.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.module.self_import(
            handler,
            engines,
            src,
            &self.mod_path,
            alias,
            is_absolute,
        )
    }

    /// Import into this namespace a single `item` from a `src` module.
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.module.item_import(
            handler,
            engines,
            src,
            item,
            &self.mod_path,
            alias,
            is_absolute,
        )
    }

    /// Import into this namespace a single variant `variant` of the enum `enum_name` from the given `src` module.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        enum_name: &Ident,
        variant_name: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.module.variant_import(
            handler,
            engines,
            src,
            enum_name,
            variant_name,
            &self.mod_path,
            alias,
            is_absolute,
        )
    }
}
