use std::{marker::PhantomData, sync::RwLock};

#[derive(Debug, Default)]
pub(crate) struct ConcurrentSlab<I, T> {
    indexer: PhantomData<I>,
    inner: RwLock<Vec<T>>,
}

impl<I, T> ConcurrentSlab<I, T>
where
    T: Clone + PartialEq,
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

    pub fn replace(&self, index: I, prev_value: &T, new_value: T) -> Option<T> {
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

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        *inner = Vec::new();
    }

    pub fn exists<F: Fn(&T) -> bool>(&self, f: F) -> bool {
        let inner = self.inner.read().unwrap();
        inner.iter().any(f)
    }
}
