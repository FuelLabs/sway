use crate::{ModuleId, SourceId};
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::RwLock,
};

/// The Source Engine manages a relationship between file paths and their corresponding
/// integer-based source IDs. Additionally, it maintains a reserve - a map that traces
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

impl SourceEngine {
    /// This function retrieves an integer-based source ID for a provided path buffer.
    /// If an ID already exists for the given path, the function will return that
    /// existing ID. If not, a new ID will be created.
    pub fn get_source_id(&self, path: &PathBuf) -> SourceId {
        {
            let source_map = self.path_to_source_map.read().unwrap();
            if source_map.contains_key(path) {
                return source_map.get(path).cloned().unwrap();
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
        self.path_to_module_map.read().unwrap().get(path).cloned()
    }
}
