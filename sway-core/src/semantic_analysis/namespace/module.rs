use crate::{
    engine_threading::Engines,
    language::{
        ty::{self},
        Visibility,
    },
    Ident, TypeId,
};

use super::{
    lexical_scope::{Items, LexicalScope, ResolvedFunctionDecl},
    LexicalScopeId, ModuleName, ModulePath, ModulePathBuf, ResolvedDeclaration,
    ResolvedTraitImplItem, TraitMap,
};

use rustc_hash::FxHasher;
use std::{collections::HashMap, hash::BuildHasherDefault};
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
    submodules: im::HashMap<ModuleName, Module, BuildHasherDefault<FxHasher>>,
    /// Keeps all lexical scopes associated with this module.
    pub lexical_scopes: Vec<LexicalScope>,
    /// Current lexical scope id in the lexical scope hierarchy stack.
    pub current_lexical_scope_id: LexicalScopeId,
    /// Maps between a span and the corresponding lexical scope id.
    pub lexical_scopes_spans: HashMap<Span, LexicalScopeId>,
    /// Name of the module, package name for root module, module name for other modules.
    /// Module name used is the same as declared in `mod name;`.
    name: Ident,
    /// Whether or not this is a `pub` module
    visibility: Visibility,
    /// Empty span at the beginning of the file implementing the module
    span: Option<Span>,
    /// An absolute path from the `root` that represents the module location.
    ///
    /// The path of the root module in a package is `[package_name]`. If a module `X` is a submodule
    /// of module `Y` which is a submodule of the root module in the package `P`, then the path is
    /// `[P, Y, X]`.
    mod_path: ModulePathBuf,
}

impl Module {
    pub(super) fn new(
        name: Ident,
        visibility: Visibility,
        span: Option<Span>,
        parent_mod_path: &ModulePathBuf,
    ) -> Self {
        let mut mod_path = parent_mod_path.clone();
        mod_path.push(name.clone());
        Self {
            visibility,
            submodules: Default::default(),
            lexical_scopes: vec![LexicalScope::default()],
            lexical_scopes_spans: Default::default(),
            current_lexical_scope_id: 0,
            name,
            span,
            mod_path,
        }
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    pub fn span(&self) -> &Option<Span> {
        &self.span
    }

    pub fn set_span(&mut self, span: Span) {
        self.span = Some(span);
    }

    pub(super) fn add_new_submodule(
        &mut self,
        name: &Ident,
        visibility: Visibility,
        span: Option<Span>,
    ) {
        let module = Self::new(name.clone(), visibility, span, &self.mod_path);
        self.submodules.insert(name.to_string(), module);
    }

    pub(crate) fn import_cached_submodule(&mut self, name: &Ident, module: Module) {
        self.submodules.insert(name.to_string(), module);
    }

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

    pub fn has_submodule(&self, name: &Ident) -> bool {
        self.submodule(&[name.clone()]).is_some()
    }

    /// Mutable access to this module's submodules.
    pub fn submodules_mut(
        &mut self,
    ) -> &mut im::HashMap<ModuleName, Module, BuildHasherDefault<FxHasher>> {
        &mut self.submodules
    }

    /// Lookup the submodule at the given path.
    pub fn submodule(&self, path: &ModulePath) -> Option<&Module> {
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
    pub fn submodule_mut(&mut self, path: &ModulePath) -> Option<&mut Module> {
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
        path: &[Ident],
    ) -> Result<&Module, ErrorEmitted> {
        match self.submodule(path) {
            None => Err(handler.emit_err(module_not_found(path, true))),
            Some(module) => Ok(module),
        }
    }

    /// Returns the root lexical scope id associated with this module.
    pub fn root_lexical_scope_id(&self) -> LexicalScopeId {
        0
    }

    /// Returns the root lexical scope associated with this module.
    pub fn root_lexical_scope(&self) -> &LexicalScope {
        self.lexical_scopes
            .get(self.root_lexical_scope_id())
            .unwrap()
    }

    pub fn get_lexical_scope(&self, id: LexicalScopeId) -> Option<&LexicalScope> {
        self.lexical_scopes.get(id)
    }

    pub fn get_lexical_scope_mut(&mut self, id: LexicalScopeId) -> Option<&mut LexicalScope> {
        self.lexical_scopes.get_mut(id)
    }

    /// Returns the current lexical scope associated with this module.
    pub fn current_lexical_scope(&self) -> &LexicalScope {
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

    /// The collection of items declared by this module's root lexical scope.
    pub fn root_items(&self) -> &Items {
        &self.root_lexical_scope().items
    }

    /// The mutable collection of items declared by this module's current lexical scope.
    pub fn current_items_mut(&mut self) -> &mut Items {
        &mut self.current_lexical_scope_mut().items
    }

    pub fn current_lexical_scope_id(&self) -> LexicalScopeId {
        self.current_lexical_scope_id
    }

    /// Enters the scope with the given span in the module's lexical scope hierarchy.
    pub fn enter_lexical_scope(
        &mut self,
        handler: &Handler,
        span: Span,
    ) -> Result<LexicalScopeId, ErrorEmitted> {
        let id_opt = self.lexical_scopes_spans.get(&span);
        match id_opt {
            Some(id) => {
                let visitor_parent = self.current_lexical_scope_id;
                self.current_lexical_scope_id = *id;
                self.current_lexical_scope_mut().visitor_parent = Some(visitor_parent);

                Ok(self.current_lexical_scope_id)
            }
            None => Err(handler.emit_err(CompileError::Internal(
                "Could not find a valid lexical scope for this source location.",
                span.clone(),
            ))),
        }
    }

    /// Pushes a new scope to the module's lexical scope hierarchy.
    pub fn push_new_lexical_scope(
        &mut self,
        span: Span,
        declaration: Option<ResolvedDeclaration>,
    ) -> LexicalScopeId {
        let previous_scope_id = self.current_lexical_scope_id();
        let previous_scope = self.lexical_scopes.get(previous_scope_id).unwrap();
        let new_scoped_id = {
            self.lexical_scopes.push(LexicalScope {
                parent: Some(previous_scope_id),
                visitor_parent: Some(previous_scope_id),
                items: Items {
                    symbols_unique_while_collecting_unifications: previous_scope
                        .items
                        .symbols_unique_while_collecting_unifications
                        .clone(),
                    ..Default::default()
                },
                declaration,
                ..Default::default()
            });
            self.lexical_scopes.len() - 1
        };
        let previous_scope = self.lexical_scopes.get_mut(previous_scope_id).unwrap();
        previous_scope.children.push(new_scoped_id);
        self.current_lexical_scope_id = new_scoped_id;
        self.lexical_scopes_spans.insert(span, new_scoped_id);
        new_scoped_id
    }

    /// Pops the current scope from the module's lexical scope hierarchy.
    pub fn pop_lexical_scope(&mut self) {
        let parent_scope_id = self.current_lexical_scope().visitor_parent;
        self.current_lexical_scope_id = parent_scope_id.unwrap(); // panics if pops do not match pushes
    }

    pub fn walk_scope_chain_early_return<T>(
        &self,
        mut f: impl FnMut(&LexicalScope) -> Result<Option<T>, ErrorEmitted>,
    ) -> Result<Option<T>, ErrorEmitted> {
        let mut lexical_scope_opt = Some(self.current_lexical_scope());
        while let Some(lexical_scope) = lexical_scope_opt {
            let result = f(lexical_scope)?;
            if let Some(result) = result {
                return Ok(Some(result));
            }
            if let Some(parent_scope_id) = lexical_scope.parent {
                lexical_scope_opt = self.get_lexical_scope(parent_scope_id);
            } else {
                lexical_scope_opt = None;
            }
        }
        Ok(None)
    }

    pub fn walk_scope_chain(&self, mut f: impl FnMut(&LexicalScope)) {
        let mut lexical_scope_opt = Some(self.current_lexical_scope());
        while let Some(lexical_scope) = lexical_scope_opt {
            f(lexical_scope);
            if let Some(parent_scope_id) = lexical_scope.parent {
                lexical_scope_opt = self.get_lexical_scope(parent_scope_id);
            } else {
                lexical_scope_opt = None;
            }
        }
    }

    pub fn append_items_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
        items: &mut Vec<ResolvedTraitImplItem>,
    ) {
        TraitMap::append_items_for_type(self, engines, type_id, items)
    }

    pub fn resolve_symbol(
        &self,
        handler: &Handler,
        engines: &Engines,
        symbol: &Ident,
    ) -> Result<(ResolvedDeclaration, ModulePathBuf), ErrorEmitted> {
        let mut last_handler = Handler::default();
        let ret = self.walk_scope_chain_early_return(|lexical_scope| {
            last_handler = Handler::default();
            Ok(lexical_scope
                .items
                .resolve_symbol(&last_handler, engines, symbol, &self.mod_path)
                .ok()
                .flatten())
        })?;

        handler.append(last_handler);

        if let Some(ret) = ret {
            Ok(ret)
        } else {
            // Symbol not found
            Err(handler.emit_err(CompileError::SymbolNotFound {
                name: symbol.clone(),
                span: symbol.span(),
            }))
        }
    }

    pub fn get_methods_for_type(
        &self,
        engines: &Engines,
        type_id: TypeId,
    ) -> Vec<ResolvedFunctionDecl> {
        let mut items = vec![];
        self.append_items_for_type(engines, type_id, &mut items);

        items.into_iter()
            .filter_map(|item| match item {
                ResolvedTraitImplItem::Parsed(_) => unreachable!(),
                ResolvedTraitImplItem::Typed(item) => match item {
                    ty::TyTraitItem::Fn(decl_ref) => Some(ResolvedFunctionDecl::Typed(decl_ref)),
                    ty::TyTraitItem::Constant(_decl_ref) => None,
                    ty::TyTraitItem::Type(_decl_ref) => None,
                },
            })
            .collect::<Vec<_>>()
    }
}

/// Create a ModuleNotFound error.
/// If skip_package_name is true, then the package name is not emitted as part of the error
/// message. This is used when the module was supposed to be found in the current package rather
/// than in an external one.
pub fn module_not_found(path: &[Ident], skip_package_name: bool) -> CompileError {
    CompileError::ModuleNotFound {
        span: path
            .iter()
            .skip(if skip_package_name { 1 } else { 0 })
            .fold(path.last().unwrap().span(), |acc, this_one| {
                if acc.source_id() == this_one.span().source_id() {
                    Span::join(acc, &this_one.span())
                } else {
                    acc
                }
            }),
        name: path
            .iter()
            .skip(if skip_package_name { 1 } else { 0 })
            .map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join("::"),
    }
}
