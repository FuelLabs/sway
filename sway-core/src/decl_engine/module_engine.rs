use std::sync::{Arc, RwLock};

use crate::{concurrent_slab_mut::ConcurrentSlabMut, namespace::Module};

/// A identifier to uniquely refer to our namespace modules.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd, Debug)]
pub struct ModuleId(usize);

impl ModuleId {
    pub fn new(index: usize) -> Self {
        ModuleId(index)
    }

    /// Returns the index that identifies the type.
    pub fn index(&self) -> usize {
        self.0
    }

    pub(crate) fn get(&self, engines: &crate::Engines) -> Arc<RwLock<Module>> {
        engines.me().get(self)
    }

    pub fn read<R>(&self, engines: &crate::Engines, mut f: impl FnMut(&Module) -> R) -> R {
        let value = self.get(engines);
        let value2 = value.read().unwrap();
        f(&value2)
    }

    pub fn write<R>(&self, engines: &crate::Engines, mut f: impl FnMut(&mut Module) -> R) -> R {
        let value = self.get(engines);
        let mut value2 = value.write().unwrap();
        f(&mut value2)
    }
}

/// The Module Engine manages a relationship between module ids and their corresponding
/// module structures.
/// The Module Engine is designed to be thread-safe. Its internal structures are
/// secured by the RwLock mechanism. This allows its functions to be invoked using
/// a straightforward non-mutable reference, ensuring safe concurrent access.
#[derive(Debug, Default, Clone)]
pub struct ModuleEngine {
    slab: ConcurrentSlabMut<Module>,
}

impl ModuleEngine {
    /// This function provides the namespace module corresponding to a specified module ID.
    pub fn get(&self, module_id: &ModuleId) -> Arc<RwLock<Module>> {
        self.slab.get(module_id.index())
    }

    pub fn read<R>(&self, module_id: &ModuleId, f: impl Fn(&Module) -> R) -> R {
        let value = self.slab.get(module_id.index());
        let value2 = value.read().unwrap();
        f(&value2)
    }

    pub fn write<R>(&self, module_id: &ModuleId, f: impl Fn(&mut Module) -> R) -> R {
        let value = self.slab.get(module_id.index());
        let mut value2 = value.write().unwrap();
        f(&mut value2)
    }

    pub fn insert(&self, value: Module) -> ModuleId {
        let id = ModuleId(self.slab.insert(value));
        self.write(&id, |m| {
            m.id = Some(id);
        });
        id
    }
}
