use std::{marker::PhantomData, sync::RwLock};

use crate::{
    declaration_engine::declaration_engine::DeclarationEngine, type_system::TypeId,
    types::ToCompileWrapper, TypeInfo,
};

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<I, T> {
    indexer: PhantomData<I>,
    inner: RwLock<Vec<T>>,
}

impl<I, T> Default for ConcurrentSlab<I, T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            indexer: PhantomData,
            inner: Default::default(),
        }
    }
}

impl<I, T> ConcurrentSlab<I, T>
where
    T: Clone,
    I: From<usize> + std::ops::Deref<Target = usize>,
{
    pub fn insert(&self, value: T) -> I {
        let mut inner = self.inner.write().unwrap();
        let ret = inner.len();
        inner.push(value);
        ret.into()
    }

    pub fn get(&self, index: I) -> T {
        let inner = self.inner.read().unwrap();
        inner[*index].clone()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        *inner = Vec::new();
    }

    pub fn exists<F: Fn(&T) -> bool>(&self, f: F) -> bool {
        let inner = self.inner.read().unwrap();
        inner.iter().any(f)
    }
}

impl ConcurrentSlab<TypeId, TypeInfo> {
    pub fn replace(
        &self,
        index: TypeId,
        prev_value: &TypeInfo,
        new_value: TypeInfo,
        declaration_engine: &DeclarationEngine,
    ) -> Option<TypeInfo> {
        // The comparison below ends up calling functions in the slab, which
        // can lead to deadlocks if we used a single read/write lock.
        // So we split the operation: we do the read only operations with
        // a single scoped read lock below, and only after the scope do
        // we get a write lock for writing into the slab.
        {
            let inner = self.inner.read().unwrap();
            let actual_prev_value = &inner[*index];
            if actual_prev_value.wrap(declaration_engine) != prev_value.wrap(declaration_engine) {
                return Some(actual_prev_value.clone());
            }
        }

        let mut inner = self.inner.write().unwrap();
        inner[*index] = new_value;
        None
    }
}
