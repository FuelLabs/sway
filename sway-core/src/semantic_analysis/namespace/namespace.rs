use crate::{
    semantic_analysis::ast_node::TypedExpression, type_system::*, CallPath, CompileResult, Ident,
    TypedDeclaration, TypedFunctionDeclaration,
};

use super::{module::Module, root::Root, submodule_namespace::SubmoduleNamespace, Path, PathBuf};

use sway_types::span::Span;

use std::collections::VecDeque;

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug, PartialEq)]
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
    pub(crate) root: Root,
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
    pub fn find_module_path<'a, T>(&'a self, prefixes: T) -> PathBuf
    where
        T: IntoIterator<Item = &'a Ident>,
    {
        self.mod_path.iter().chain(prefixes).cloned().collect()
    }

    /// A reference to the root of the project namespace.
    pub fn root(&self) -> &Root {
        &self.root
    }

    /// A mutable reference to the root of the project namespace.
    pub fn root_mut(&mut self) -> &mut Root {
        &mut self.root
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
    pub(crate) fn resolve_symbol(&self, symbol: &Ident) -> CompileResult<&TypedDeclaration> {
        self.root.resolve_symbol(&self.mod_path, symbol)
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        call_path: &CallPath,
    ) -> CompileResult<&TypedDeclaration> {
        self.root.resolve_call_path(&self.mod_path, call_path)
    }

    /// Short-hand for calling [Root::resolve_type_with_self] on `root` with the `mod_path`.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        type_id: TypeId,
        self_type: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        self.root.resolve_type_with_self(
            type_id,
            self_type,
            span,
            enforce_type_arguments,
            type_info_prefix,
            &self.mod_path,
        )
    }

    /// Short-hand for calling [Root::resolve_type_without_self] on `root` and with the `mod_path`.
    pub(crate) fn resolve_type_without_self(
        &mut self,
        type_id: TypeId,
        span: &Span,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        self.root.resolve_type(
            type_id,
            span,
            EnforceTypeArguments::Yes,
            type_info_prefix,
            &self.mod_path,
        )
    }

    /// Short-hand for calling [Root::find_method_for_type] on `root` with the `mod_path`.
    pub(crate) fn find_method_for_type(
        &mut self,
        r#type: TypeId,
        method_prefix: &Path,
        method_name: &Ident,
        args_buf: &VecDeque<TypedExpression>,
    ) -> CompileResult<TypedFunctionDeclaration> {
        self.root
            .find_method_for_type(&self.mod_path, r#type, method_prefix, method_name, args_buf)
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(&mut self, src: &Path) -> CompileResult<()> {
        self.root.star_import(src, &self.mod_path)
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(&mut self, src: &Path, alias: Option<Ident>) -> CompileResult<()> {
        self.root.self_import(src, &self.mod_path, alias)
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        self.root.item_import(src, item, &self.mod_path, alias)
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(&mut self, dep_name: Ident) -> SubmoduleNamespace {
        let init = self.init.clone();
        self.submodules.entry(dep_name.to_string()).or_insert(init);
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(dep_name))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }
}

impl std::ops::Deref for Namespace {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl std::ops::DerefMut for Namespace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
