use std::sync::{Arc, RwLock};

use crate::{concurrent_slab_mut::ConcurrentSlabMut, engine_threading::DebugWithEngines};

use super::ParseModule;

/// A identifier to uniquely refer to our parsed modules.
#[derive(Default, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug)]
pub struct ParseModuleId(usize);

impl ParseModuleId {
    pub fn new(index: usize) -> Self {
        ParseModuleId(index)
    }

    /// Returns the index that identifies the type.
    pub fn index(&self) -> usize {
        self.0
    }

    pub(crate) fn get(&self, engines: &crate::Engines) -> Arc<RwLock<ParseModule>> {
        engines.pme().get(self)
    }

    pub fn read<R>(&self, engines: &crate::Engines, f: impl Fn(&ParseModule) -> R) -> R {
        let value = self.get(engines);
        let value = value.read().unwrap();
        f(&value)
    }

    pub fn write<R>(
        &self,
        engines: &crate::Engines,
        mut f: impl FnMut(&mut ParseModule) -> R,
    ) -> R {
        let value = self.get(engines);
        let mut value = value.write().unwrap();
        f(&mut value)
    }
}

impl DebugWithEngines for ParseModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, engines: &crate::Engines) -> std::fmt::Result {
        let name = self.read(engines, |m| m.name.clone());
        write!(f, "{:?}", name)
    }
}

/// The Parsed Module Engine manages a relationship between module ids and their corresponding
/// parsed module structures.
#[derive(Debug, Default, Clone)]
pub struct ParsedModuleEngine {
    slab: ConcurrentSlabMut<ParseModule>,
}

impl ParsedModuleEngine {
    /// This function provides the namespace module corresponding to a specified module ID.
    pub fn get(&self, module_id: &ParseModuleId) -> Arc<RwLock<ParseModule>> {
        self.slab.get(module_id.index())
    }

    pub fn read<R>(&self, module_id: &ParseModuleId, f: impl Fn(&ParseModule) -> R) -> R {
        let value = self.slab.get(module_id.index());
        let value = value.read().unwrap();
        f(&value)
    }

    pub fn write<R>(&self, module_id: &ParseModuleId, f: impl Fn(&mut ParseModule) -> R) -> R {
        let value = self.slab.get(module_id.index());
        let mut value = value.write().unwrap();
        f(&mut value)
    }

    pub fn insert(&self, value: ParseModule) -> ParseModuleId {
        let id = ParseModuleId(self.slab.insert(value));
        self.write(&id, |m| {
            m.id = id;
        });
        id
    }
}
