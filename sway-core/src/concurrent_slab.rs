use std::sync::RwLock;

use crate::{type_system::TypeId, TypeInfo};

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<T> {
    inner: RwLock<Vec<T>>,
}

impl<T> Default for ConcurrentSlab<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T> ConcurrentSlab<T>
where
    T: Clone,
{
    pub fn insert(&self, value: T) -> usize {
        let mut inner = self.inner.write().unwrap();
        let ret = inner.len();
        inner.push(value);
        ret
    }

    pub fn get(&self, index: usize) -> T {
        let inner = self.inner.read().unwrap();
        inner[index].clone()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        *inner = Vec::new();
    }

    pub fn exists<F: Fn(&T) -> bool>(&self, f: F) -> bool {
        let inner = self.inner.read().unwrap();
        inner.iter().any(f)
    }

    pub fn size(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.len()
    }

    pub fn blind_replace(&self, index: usize, new_value: T) {
        let mut inner = self.inner.write().unwrap();
        inner[index] = new_value;
    }
}

impl ConcurrentSlab<TypeInfo> {
    pub fn replace(
        &self,
        index: TypeId,
        prev_value: &TypeInfo,
        new_value: TypeInfo,
    ) -> Option<TypeInfo> {
        // The comparison below ends up calling functions in the slab, which
        // can lead to deadlocks if we used a single read/write lock.
        // So we split the operation: we do the read only operations with
        // a single scoped read lock below, and only after the scope do
        // we get a write lock for writing into the slab.
        {
            let inner = self.inner.read().unwrap();
            let actual_prev_value = &inner[*index];
            if actual_prev_value != prev_value {
                return Some(actual_prev_value.clone());
            }
        }

        let mut inner = self.inner.write().unwrap();
        inner[*index] = new_value;
        None
    }
}
