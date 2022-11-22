use std::{fmt, sync::RwLock};

use crate::{
    declaration_engine::{declaration_id::DeclarationId, declaration_wrapper::DeclarationWrapper},
    engine_threading::*,
    type_system::TypeId,
    TypeEngine, TypeInfo,
};

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

impl<T> ConcurrentSlab<T> {
    pub fn with_slice<R>(&self, run: impl FnOnce(&[T]) -> R) -> R {
        run(&self.inner.read().unwrap())
    }
}

pub struct ListDisplay<I> {
    pub list: I,
}

impl<I: IntoIterator + Clone> fmt::Display for ListDisplay<I>
where
    I::Item: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_elems = self
            .list
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, value)| format!("{:<10}\t->\t{}", i, value))
            .collect::<Vec<_>>();
        write!(f, "{}", fmt_elems.join("\n"))
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
}

impl ConcurrentSlab<TypeInfo> {
    pub fn replace(
        &self,
        index: TypeId,
        prev_value: &TypeInfo,
        new_value: TypeInfo,
        type_engine: &TypeEngine,
    ) -> Option<TypeInfo> {
        let index = index.index();
        // The comparison below ends up calling functions in the slab, which
        // can lead to deadlocks if we used a single read/write lock.
        // So we split the operation: we do the read only operations with
        // a single scoped read lock below, and only after the scope do
        // we get a write lock for writing into the slab.
        {
            let inner = self.inner.read().unwrap();
            let actual_prev_value = &inner[index];
            if !actual_prev_value.eq(prev_value, type_engine) {
                return Some(actual_prev_value.clone());
            }
        }

        let mut inner = self.inner.write().unwrap();
        inner[index] = new_value;
        None
    }
}

impl ConcurrentSlab<DeclarationWrapper> {
    pub fn replace(
        &self,
        index: DeclarationId,
        new_value: DeclarationWrapper,
    ) -> Option<DeclarationWrapper> {
        let mut inner = self.inner.write().unwrap();
        inner[*index] = new_value;
        None
    }
}
