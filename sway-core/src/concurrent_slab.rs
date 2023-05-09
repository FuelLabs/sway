use std::{fmt, sync::RwLock};

use sway_types::{Named, Spanned};

use crate::{decl_engine::*, engine_threading::*, type_system::*};

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<T> {
    inner: RwLock<Vec<T>>,
}

impl<T> Clone for ConcurrentSlab<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap();
        Self {
            inner: RwLock::new(inner.clone()),
        }
    }
}

impl<T> Default for ConcurrentSlab<T> {
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
            .map(|(i, value)| format!("{i:<10}\t->\t{value}"))
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
}

impl ConcurrentSlab<TypeInfo> {
    pub fn replace(
        &self,
        index: TypeId,
        prev_value: &TypeInfo,
        new_value: TypeInfo,
        engines: Engines<'_>,
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
            if !actual_prev_value.eq(prev_value, engines) {
                return Some(actual_prev_value.clone());
            }
        }

        let mut inner = self.inner.write().unwrap();
        inner[index] = new_value;
        None
    }
}

impl<T> ConcurrentSlab<T>
where
    DeclEngine: DeclEngineIndex<T>,
    T: Named + Spanned,
{
    pub fn replace(&self, index: DeclId<T>, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        inner[index.inner()] = new_value;
        None
    }
}
