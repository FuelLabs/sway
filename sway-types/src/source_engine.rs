use std::{collections::HashMap, path::PathBuf, sync::RwLock};

use crate::SourceId;

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
    next_id: RwLock<u32>,
    source_map: RwLock<HashMap<PathBuf, SourceId>>,
    path_map: RwLock<HashMap<SourceId, PathBuf>>,
}

impl SourceEngine {
    /// This function retrieves an integer-based source ID for a provided path buffer.
    /// If an ID already exists for the given path, the function will return that
    /// existing ID. If not, a new ID will be created.
    pub fn get_source_id(&self, path: &PathBuf) -> SourceId {
        {
            let source_map = self.source_map.read().unwrap();
            if source_map.contains_key(path) {
                return source_map.get(path).cloned().unwrap();
            }
        }

        let source_id = SourceId {
            id: *self.next_id.read().unwrap(),
        };
        {
            let mut next_id = self.next_id.write().unwrap();
            *next_id += 1;

            let mut source_map = self.source_map.write().unwrap();
            source_map.insert(path.clone(), source_id);

            let mut path_map = self.path_map.write().unwrap();
            path_map.insert(source_id, path.clone());
        }

        source_id
    }

    /// This function provides the file path corresponding to a specified source ID.
    pub fn get_path(&self, source_id: &SourceId) -> PathBuf {
        self.path_map
            .read()
            .unwrap()
            .get(source_id)
            .unwrap()
            .clone()
    }
}
