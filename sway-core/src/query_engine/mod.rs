use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};
use sway_error::{error::CompileError, warning::CompileWarning};
use sway_types::{IdentUnique, ProgramId, SourceId, Spanned};

use crate::{
    decl_engine::{DeclId, DeclRef},
    language::ty::{TyFunctionDecl, TyFunctionSig, TyModule},
    namespace, Engines, Programs,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ModuleCacheKey {
    pub path: Arc<PathBuf>,
    pub include_tests: bool,
}

impl ModuleCacheKey {
    pub fn new(path: Arc<PathBuf>, include_tests: bool) -> Self {
        Self {
            path,
            include_tests,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModuleCommonInfo {
    pub path: Arc<PathBuf>,
    pub hash: u64,
    pub include_tests: bool,
    pub dependencies: Vec<Arc<PathBuf>>,
}

#[derive(Clone, Debug)]
pub struct ParsedModuleInfo {
    pub modified_time: Option<SystemTime>,
    pub version: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct TypedModuleInfo {
    pub module: Arc<TyModule>,
    pub namespace_module: Arc<namespace::Module>,
    pub version: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ModuleCacheEntry {
    pub common: ModuleCommonInfo,
    pub parsed: ParsedModuleInfo,
    pub typed: Option<TypedModuleInfo>,
}

impl ModuleCacheEntry {
    pub fn new(common: ModuleCommonInfo, parsed: ParsedModuleInfo) -> Self {
        Self {
            common,
            parsed,
            typed: None,
        }
    }

    pub fn is_typed(&self) -> bool {
        self.typed.is_some()
    }

    pub fn set_typed(&mut self, typed: TypedModuleInfo) {
        self.typed = Some(typed);
    }

    pub fn update_common(&mut self, new_common: ModuleCommonInfo) {
        self.common = new_common;
    }

    pub fn update_parsed(&mut self, new_parsed: ParsedModuleInfo) {
        self.parsed = new_parsed;
    }

    pub fn update_parsed_and_common(
        &mut self,
        new_common: ModuleCommonInfo,
        new_parsed: ParsedModuleInfo,
    ) {
        self.common = new_common;
        self.parsed = new_parsed;
    }
}

#[derive(Debug, Default, Clone)]
pub struct ModuleCacheMap(HashMap<ModuleCacheKey, ModuleCacheEntry>);

impl Deref for ModuleCacheMap {
    type Target = HashMap<ModuleCacheKey, ModuleCacheEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ModuleCacheMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ModuleCacheMap {
    pub fn update_entry(
        &mut self,
        key: &ModuleCacheKey,
        new_common: ModuleCommonInfo,
        new_parsed: ParsedModuleInfo,
    ) {
        if let Some(entry) = self.get_mut(key) {
            entry.update_parsed_and_common(new_common, new_parsed);
        } else {
            self.insert(key.clone(), ModuleCacheEntry::new(new_common, new_parsed));
        }
    }
}

pub type ProgramsCacheMap = HashMap<Arc<PathBuf>, ProgramsCacheEntry>;
pub type FunctionsCacheMap = HashMap<(IdentUnique, String), FunctionCacheEntry>;

#[derive(Clone, Debug)]
pub struct ProgramsCacheEntry {
    pub path: Arc<PathBuf>,
    pub programs: Programs,
    pub handler_data: (Vec<CompileError>, Vec<CompileWarning>),
}

#[derive(Clone, Debug)]
pub struct FunctionCacheEntry {
    pub fn_decl: DeclRef<DeclId<TyFunctionDecl>>,
}

#[derive(Debug, Default)]
pub struct QueryEngine {
    // We want the below types wrapped in Arcs to optimize cloning from LSP.
    programs_cache: CowCache<ProgramsCacheMap>,
    pub module_cache: CowCache<ModuleCacheMap>,
    // NOTE: Any further AstNodes that are cached need to have garbage collection applied, see clear_module()
    function_cache: CowCache<FunctionsCacheMap>,
}

impl Clone for QueryEngine {
    fn clone(&self) -> Self {
        Self {
            programs_cache: CowCache::new(self.programs_cache.read().clone()),
            module_cache: CowCache::new(self.module_cache.read().clone()),
            function_cache: CowCache::new(self.function_cache.read().clone()),
        }
    }
}

impl QueryEngine {
    pub fn update_or_insert_parsed_module_cache_entry(&self, entry: ModuleCacheEntry) {
        let path = entry.common.path.clone();
        let include_tests = entry.common.include_tests;
        let key = ModuleCacheKey::new(path, include_tests);
        let mut cache = self.module_cache.write();
        cache.update_entry(&key, entry.common, entry.parsed);
    }

    pub fn update_typed_module_cache_entry(&self, key: &ModuleCacheKey, entry: TypedModuleInfo) {
        let mut cache = self.module_cache.write();
        cache.get_mut(key).unwrap().set_typed(entry);
    }

    pub fn get_programs_cache_entry(&self, path: &Arc<PathBuf>) -> Option<ProgramsCacheEntry> {
        let cache = self.programs_cache.read();
        cache.get(path).cloned()
    }

    pub fn insert_programs_cache_entry(&self, entry: ProgramsCacheEntry) {
        let mut cache = self.programs_cache.write();
        cache.insert(entry.path.clone(), entry);
    }

    pub fn get_function(
        &self,
        engines: &Engines,
        ident: &IdentUnique,
        sig: TyFunctionSig,
    ) -> Option<DeclRef<DeclId<TyFunctionDecl>>> {
        // let cache = self.function_cache.read();
        // cache
        //     .get(&(ident.clone(), sig.get_type_str(engines)))
        //     .map(|s| s.fn_decl.clone())
        None
    }

    pub fn insert_function(
        &self,
        engines: &Engines,
        ident: IdentUnique,
        sig: TyFunctionSig,
        fn_decl: DeclRef<DeclId<TyFunctionDecl>>,
    ) {
        let mut cache = self.function_cache.write();
        cache.insert(
            (ident, sig.get_type_str(engines)),
            FunctionCacheEntry { fn_decl },
        );
    }

    /// Removes all data associated with the `source_id` from the function cache.
    pub fn clear_module(&mut self, source_id: &SourceId) {
        self.function_cache
            .write()
            .retain(|(ident, _), _| (ident.span().source_id() != Some(source_id)));
    }

    /// Removes all data associated with the `program_id` from the function cache.
    pub fn clear_program(&mut self, program_id: &ProgramId) {
        self.function_cache.write().retain(|(ident, _), _| {
            ident
                .span()
                .source_id()
                .is_none_or(|id| id.program_id() != *program_id)
        });
    }

    ///  Commits all changes to their respective caches.
    pub fn commit(&self) {
        self.programs_cache.commit();
        self.module_cache.commit();
        self.function_cache.commit();
    }
}

/// Thread-safe, copy-on-write cache optimized for LSP operations.
///
/// Addresses key LSP challenges:
/// 1. Concurrent read access to shared data
/// 2. Local modifications for cancellable operations (e.g., compilation)
/// 3. Prevents incomplete results from affecting shared state
/// 4. Maintains consistency via explicit commit step
///
/// Uses `Arc<RwLock<T>>` for shared state and `RwLock<Option<T>>` for local changes.
/// Suitable for interactive sessions with frequent file changes.
#[derive(Debug, Default)]
pub struct CowCache<T: Clone> {
    inner: Arc<RwLock<T>>,
    local: RwLock<Option<T>>,
}

impl<T: Clone> CowCache<T> {
    /// Creates a new `CowCache` with the given initial value.
    ///
    /// The value is wrapped in an `Arc<RwLock<T>>` to allow shared access across threads.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            local: RwLock::new(None),
        }
    }

    /// Provides read access to the cached value.
    ///
    /// If a local modification exists, it returns a reference to the local copy.
    /// Otherwise, it returns a reference to the shared state.
    ///
    /// This method is optimized for concurrent read access in LSP operations.
    pub fn read(&self) -> impl Deref<Target = T> + '_ {
        if self.local.read().is_some() {
            ReadGuard::Local(self.local.read())
        } else {
            ReadGuard::Shared(self.inner.read())
        }
    }

    /// Provides write access to a local copy of the cached value.
    ///
    /// In LSP, this is used for operations like compilation tasks that may be cancelled.
    /// It allows modifications without affecting the shared state until explicitly committed.
    pub fn write(&self) -> impl DerefMut<Target = T> + '_ {
        let mut local = self.local.write();
        if local.is_none() {
            *local = Some(self.inner.read().clone());
        }
        WriteGuard(local)
    }

    /// Commits local modifications to the shared state.
    ///
    /// Called after successful completion of a compilation task.
    /// If a task is cancelled, not calling this method effectively discards local changes.
    pub fn commit(&self) {
        if let Some(local) = self.local.write().take() {
            *self.inner.write() = local;
        }
    }
}

/// A guard type that provides read access to either the local or shared state.
enum ReadGuard<'a, T: Clone> {
    Local(RwLockReadGuard<'a, Option<T>>),
    Shared(RwLockReadGuard<'a, T>),
}

impl<T: Clone> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadGuard::Local(r) => r.as_ref().unwrap(),
            ReadGuard::Shared(guard) => guard.deref(),
        }
    }
}

/// A guard type that provides write access to the local state.
struct WriteGuard<'a, T: Clone>(RwLockWriteGuard<'a, Option<T>>);

impl<T: Clone> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: Clone> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
