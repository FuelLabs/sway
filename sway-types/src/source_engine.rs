use crate::{ModuleId, SourceId};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::RwLock,
};

/// The Source Engine manages a relationship between file paths and their corresponding
/// integer-based source IDs. Additionally, it maintains the reverse - a map that traces
/// back from a source ID to its original file path. The primary objective of this
/// system is to enable clients that need to reference a file path to do so using an
/// integer-based ID. This numeric representation can be stored more efficiently as
/// a key in a hashmap.
/// The Source Engine is designed to be thread-safe. Its internal structures are
/// secured by the RwLock mechanism. This allows its functions to be invoked using
/// a straightforward non-mutable reference, ensuring safe concurrent access.
#[derive(Debug, Default)]
pub struct SourceEngine {
    next_source_id: RwLock<u32>,
    path_to_source_map: RwLock<HashMap<PathBuf, SourceId>>,
    source_to_path_map: RwLock<HashMap<SourceId, PathBuf>>,
    next_module_id: RwLock<u16>,
    path_to_module_map: RwLock<HashMap<PathBuf, ModuleId>>,
    module_to_sources_map: RwLock<HashMap<ModuleId, BTreeSet<SourceId>>>,
}

impl Clone for SourceEngine {
    fn clone(&self) -> Self {
        SourceEngine {
            next_source_id: RwLock::new(*self.next_source_id.read().unwrap()),
            path_to_source_map: RwLock::new(self.path_to_source_map.read().unwrap().clone()),
            source_to_path_map: RwLock::new(self.source_to_path_map.read().unwrap().clone()),
            next_module_id: RwLock::new(*self.next_module_id.read().unwrap()),
            path_to_module_map: RwLock::new(self.path_to_module_map.read().unwrap().clone()),
            module_to_sources_map: RwLock::new(self.module_to_sources_map.read().unwrap().clone()),
        }
    }
}

impl SourceEngine {
    const AUTOGENERATED_PATH: &'static str = "<autogenerated>";

    pub fn is_span_in_autogenerated(&self, span: &crate::Span) -> Option<bool> {
        span.source_id().map(|s| self.is_source_id_autogenerated(s))
    }

    pub fn is_source_id_autogenerated(&self, source_id: &SourceId) -> bool {
        self.get_path(source_id).starts_with("<autogenerated>")
    }

    /// This function retrieves an integer-based source ID for a provided path buffer.
    /// If an ID already exists for the given path, the function will return that
    /// existing ID. If not, a new ID will be created.
    pub fn get_source_id(&self, path: &PathBuf) -> SourceId {
        {
            let source_map = self.path_to_source_map.read().unwrap();
            if source_map.contains_key(path) {
                return source_map.get(path).copied().unwrap();
            }
        }
        let manifest_path = sway_utils::find_parent_manifest_dir(path).unwrap_or(path.clone());
        let module_id = {
            let mut module_map = self.path_to_module_map.write().unwrap();
            *module_map.entry(manifest_path.clone()).or_insert_with(|| {
                let mut next_id = self.next_module_id.write().unwrap();
                *next_id += 1;
                ModuleId::new(*next_id)
            })
        };

        self.get_source_id_with_module_id(path, module_id)
    }

    pub fn get_source_id_with_module_id(&self, path: &PathBuf, module_id: ModuleId) -> SourceId {
        {
            let source_map = self.path_to_source_map.read().unwrap();
            if source_map.contains_key(path) {
                return source_map.get(path).copied().unwrap();
            }
        }

        let source_id = SourceId::new(module_id.id, *self.next_source_id.read().unwrap());
        {
            let mut next_id = self.next_source_id.write().unwrap();
            *next_id += 1;

            let mut source_map = self.path_to_source_map.write().unwrap();
            source_map.insert(path.clone(), source_id);

            let mut path_map = self.source_to_path_map.write().unwrap();
            path_map.insert(source_id, path.clone());
        }

        let mut module_map = self.module_to_sources_map.write().unwrap();
        module_map.entry(module_id).or_default().insert(source_id);

        source_id
    }

    pub fn get_autogenerated_source_id(&self, module_id: ModuleId) -> SourceId {
        self.get_source_id_with_module_id(&Self::AUTOGENERATED_PATH.into(), module_id)
    }

    /// This function provides the file path corresponding to a specified source ID.
    pub fn get_path(&self, source_id: &SourceId) -> PathBuf {
        self.source_to_path_map
            .read()
            .unwrap()
            .get(source_id)
            .unwrap()
            .clone()
    }

    /// This function provides the module ID corresponding to a specified file path.
    pub fn get_module_id(&self, path: &PathBuf) -> Option<ModuleId> {
        self.path_to_module_map.read().unwrap().get(path).copied()
    }

    /// Returns the [PathBuf] associated with the provided [ModuleId], if it exists in the path_to_module_map.
    pub fn get_path_from_module_id(&self, module_id: &ModuleId) -> Option<PathBuf> {
        let path_to_module_map = self.path_to_module_map.read().unwrap();
        path_to_module_map
            .iter()
            .find(|(_, &id)| id == *module_id)
            .map(|(path, _)| path.clone())
    }

    /// This function provides the file name (with extension) corresponding to a specified source ID.
    pub fn get_file_name(&self, source_id: &SourceId) -> Option<String> {
        self.get_path(source_id)
            .as_path()
            .file_name()
            .map(|file_name| file_name.to_string_lossy())
            .map(|file_name| file_name.to_string())
    }

    pub fn all_files(&self) -> Vec<PathBuf> {
        let s = self.source_to_path_map.read().unwrap();
        let mut v = s.values().cloned().collect::<Vec<_>>();
        v.sort();
        v
    }

    pub fn get_source_ids_from_module_id(&self, module_id: ModuleId) -> Option<BTreeSet<SourceId>> {
        let s = self.module_to_sources_map.read().unwrap();
        s.get(&module_id).cloned()
    }
}
