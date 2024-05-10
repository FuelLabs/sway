use crate::{engine_threading::Engines, language::Visibility, Ident};

use super::{
    lexical_scope::{Items, LexicalScope},
    root::Root,
    LexicalScopeId, ModuleName, ModulePath, ModulePathBuf,
};

use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use sway_error::handler::Handler;
use sway_error::{error::CompileError, handler::ErrorEmitted};
use sway_types::{span::Span, Spanned};

/// A single `Module` within a Sway project.
///
/// A `Module` is most commonly associated with an individual file of Sway code, e.g. a top-level
/// script/predicate/contract file or some library dependency whether introduced via `mod` or the
/// `[dependencies]` table of a `forc` manifest.
///
/// A `Module` contains a set of all items that exist within the lexical scope via declaration or
/// importing, along with a map of each of its submodules.
#[derive(Clone, Debug)]
pub struct Module {
    /// Submodules of the current module represented as an ordered map from each submodule's name
    /// to the associated `Module`.
    ///
    /// Submodules are normally introduced in Sway code with the `mod foo;` syntax where `foo` is
    /// some library dependency that we include as a submodule.
    ///
    /// Note that we *require* this map to produce deterministic codegen results which is why [`FxHasher`] is used.
    pub(crate) submodules: im::HashMap<ModuleName, Module, BuildHasherDefault<FxHasher>>,
    /// Keeps all lexical scopes associated with this module.
    pub lexical_scopes: Vec<LexicalScope>,
    /// Current lexical scope id in the lexical scope hierarchy stack.
    pub current_lexical_scope_id: LexicalScopeId,
    /// Name of the module, package name for root module, module name for other modules.
    /// Module name used is the same as declared in `mod name;`.
    pub name: Option<Ident>,
    /// Whether or not this is a `pub` module
    pub visibility: Visibility,
    /// Empty span at the beginning of the file implementing the module
    pub span: Option<Span>,
    /// Indicates whether the module is external to the current package. External modules are
    /// imported in the `Forc.toml` file.
    pub is_external: bool,
    /// An absolute path from the `root` that represents the module location.
    ///
    /// When this is the root module, this is equal to `[]`. When this is a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub(crate) mod_path: ModulePathBuf,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            visibility: Visibility::Private,
            submodules: Default::default(),
            lexical_scopes: vec![LexicalScope::default()],
            current_lexical_scope_id: 0,
            name: Default::default(),
            span: Default::default(),
            is_external: Default::default(),
            mod_path: Default::default(),
        }
    }
}

impl Module {
    pub fn read<R>(&self, _engines: &crate::Engines, mut f: impl FnMut(&Module) -> R) -> R {
        f(self)
    }

    pub fn write<R>(
        &mut self,
        _engines: &crate::Engines,
        mut f: impl FnMut(&mut Module) -> R,
    ) -> R {
        f(self)
    }

    pub fn mod_path(&self) -> &ModulePath {
        self.mod_path.as_slice()
    }

    pub fn mod_path_buf(&self) -> ModulePathBuf {
        self.mod_path.clone()
    }

    /// Immutable access to this module's submodules.
    pub fn submodules(&self) -> &im::HashMap<ModuleName, Module, BuildHasherDefault<FxHasher>> {
        &self.submodules
    }

    /// Insert a submodule into this `Module`.
    pub fn insert_submodule(&mut self, name: String, submodule: Module) {
        self.submodules.insert(name, submodule);
    }

    /// Lookup the submodule at the given path.
    pub fn submodule(&self, _engines: &Engines, path: &ModulePath) -> Option<&Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Unique access to the submodule at the given path.
    pub fn submodule_mut(&mut self, _engines: &Engines, path: &ModulePath) -> Option<&mut Module> {
        let mut module = self;
        for ident in path.iter() {
            match module.submodules.get_mut(ident.as_str()) {
                Some(ns) => module = ns,
                None => return None,
            }
        }
        Some(module)
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn lookup_submodule(
        &self,
        handler: &Handler,
        engines: &Engines,
        path: &[Ident],
    ) -> Result<&Module, ErrorEmitted> {
        match self.submodule(engines, path) {
            None => Err(handler.emit_err(module_not_found(path))),
            Some(module) => Ok(module),
        }
    }

    /// Lookup the submodule at the given path.
    ///
    /// This should be used rather than `Index` when we don't yet know whether the module exists.
    pub(crate) fn lookup_submodule_mut(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        path: &[Ident],
    ) -> Result<&mut Module, ErrorEmitted> {
        match self.submodule_mut(engines, path) {
            None => Err(handler.emit_err(module_not_found(path))),
            Some(module) => Ok(module),
        }
    }

    /// Returns the current lexical scope associated with this module.
    fn current_lexical_scope(&self) -> &LexicalScope {
        self.lexical_scopes
            .get(self.current_lexical_scope_id)
            .unwrap()
    }

    /// Returns the mutable current lexical scope associated with this module.
    pub fn current_lexical_scope_mut(&mut self) -> &mut LexicalScope {
        self.lexical_scopes
            .get_mut(self.current_lexical_scope_id)
            .unwrap()
    }

    /// The collection of items declared by this module's current lexical scope.
    pub fn current_items(&self) -> &Items {
        &self.current_lexical_scope().items
    }

    /// The mutable collection of items declared by this module's current lexical scope.
    pub fn current_items_mut(&mut self) -> &mut Items {
        &mut self.current_lexical_scope_mut().items
    }

    pub fn current_lexical_scope_id(&self) -> LexicalScopeId {
        self.current_lexical_scope_id
    }

    /// Pushes a new scope to the module's lexical scope hierarchy.
    pub fn push_new_lexical_scope(&mut self) -> LexicalScopeId {
        let previous_scope_id = self.current_lexical_scope_id();
        let new_scoped_id = {
            self.lexical_scopes.push(LexicalScope {
                parent: Some(previous_scope_id),
                ..Default::default()
            });
            self.current_lexical_scope_id()
        };
        let previous_scope = self.lexical_scopes.get_mut(previous_scope_id).unwrap();
        previous_scope.children.push(new_scoped_id);
        self.current_lexical_scope_id = new_scoped_id;
        new_scoped_id
    }

    /// Pops the current scope from the module's lexical scope hierarchy.
    pub fn pop_lexical_scope(&mut self) {
        let parent_scope_id = self.current_lexical_scope().parent;
        self.current_lexical_scope_id = parent_scope_id.unwrap_or(0);
    }
}

impl From<Root> for Module {
    fn from(root: Root) -> Self {
        root.module
    }
}

fn module_not_found(path: &[Ident]) -> CompileError {
    CompileError::ModuleNotFound {
        span: path.iter().fold(path[0].span(), |acc, this_one| {
            if acc.source_id() == this_one.span().source_id() {
                Span::join(acc, &this_one.span())
            } else {
                acc
            }
        }),
        name: path
            .iter()
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}
