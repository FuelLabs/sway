use parking_lot::RwLock;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
    time::SystemTime,
};
use sway_error::{error::CompileError, warning::CompileWarning};
use sway_types::IdentUnique;

use crate::{
    decl_engine::{DeclId, DeclRef},
    language::ty::{TyFunctionDecl, TyFunctionSig, TyModule},
    {Engines, Programs},
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
    pub module: TyModule,
    pub modified_time: Option<SystemTime>,
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
struct ModuleCacheMap(HashMap<ModuleCacheKey, ModuleCacheEntry>);

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

#[derive(Debug, Default, Clone)]
pub struct QueryEngine {
    // We want the below types wrapped in Arcs to optimize cloning from LSP.
    module_cache: Arc<RwLock<ModuleCacheMap>>,
    programs_cache: Arc<RwLock<ProgramsCacheMap>>,
    function_cache: Arc<RwLock<FunctionsCacheMap>>,
}

impl QueryEngine {
    pub fn get_module_cache_entry(&self, key: &ModuleCacheKey) -> Option<ModuleCacheEntry> {
        let cache = self.module_cache.read();
        cache.get(key).cloned()
    }

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
        ident: IdentUnique,
        sig: TyFunctionSig,
    ) -> Option<DeclRef<DeclId<TyFunctionDecl>>> {
        let cache = self.function_cache.read();
        cache
            .get(&(ident, sig.get_type_str(engines)))
            .map(|s| s.fn_decl.clone())
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
}
