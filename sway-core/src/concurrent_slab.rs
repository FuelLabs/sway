use parking_lot::RwLock;
use std::{fmt, sync::Arc};
use sway_utils::ConcurrentSlabMetrics;

#[derive(Debug, Clone)]
pub struct Inner<T> {
    pub items: Vec<Option<Arc<T>>>,
    pub free_list: Vec<usize>,
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
            free_list: Default::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<T> {
    pub inner: RwLock<Inner<T>>,
}

impl<T> Clone for ConcurrentSlab<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.read();
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
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let inner = self.inner.read();
        inner.items.len()
    }

    pub fn values(&self) -> Vec<Arc<T>> {
        let inner = self.inner.read();
        inner.items.iter().filter_map(|x| x.clone()).collect()
    }

    pub fn insert(&self, value: T) -> usize {
        self.insert_arc(Arc::new(value))
    }

    pub fn insert_arc(&self, value: Arc<T>) -> usize {
        let mut inner = self.inner.write();
        if let Some(free) = inner.free_list.pop() {
            assert!(inner.items[free].is_none());
            inner.items[free] = Some(value);
            free
        } else {
            inner.items.push(Some(value));
            inner.items.len() - 1
        }
    }

    /// Reserves an empty slot and returns its index.
    ///
    /// The reserved slot contains no value but is **not on the free list**.
    /// Accessing it via [Self::get] or [Self::map] panics until the slot
    /// is filled via [Self::fill].
    pub fn reserve(&self) -> usize {
        let mut inner = self.inner.write();
        if let Some(free) = inner.free_list.pop() {
            assert!(inner.items[free].is_none());
            free
        } else {
            inner.items.push(None);
            inner.items.len() - 1
        }
    }

    /// Fills the slot at `index` with the `value`.
    ///
    /// Unlike [Self::replace], the slot is allowed to be empty (reserved
    /// via [Self::reserve] and not yet filled).
    pub fn fill(&self, index: usize, value: T) {
        let mut inner = self.inner.write();
        let item = inner
            .items
            .get_mut(index)
            .expect("invalid slab index for ConcurrentSlab::fill");
        *item = Some(Arc::new(value));
    }

    /// Returns true if the slot at `index` is filled.
    pub fn is_filled(&self, index: usize) -> bool {
        let inner = self.inner.read();
        inner
            .items
            .get(index)
            .expect("invalid slab index for ConcurrentSlab::is_filled")
            .is_some()
    }

    /// Releases a slot reserved via [Self::reserve] that was never filled,
    /// returning it to the free list.
    ///
    /// Panics if called on a filled slot.
    pub fn release_reserved(&self, index: usize) {
        let mut inner = self.inner.write();
        assert!(
            inner.items[index].is_none(),
            "only an empty (reserved and not filled) slot can be released"
        );
        assert!(
            !inner.free_list.contains(&index),
            "the reserved slot must not be on the free list"
        );
        inner.free_list.push(index);
    }

    pub fn replace(&self, index: usize, new_value: T) -> Option<T> {
        let mut inner = self.inner.write();
        let item = inner.items.get_mut(index)?;
        let old = item.replace(Arc::new(new_value))?;
        Arc::into_inner(old)
    }

    pub fn replace_arc(&self, index: usize, new_value: Arc<T>) -> Option<T> {
        let mut inner = self.inner.write();
        let item = inner.items.get_mut(index)?;
        let old = item.replace(new_value)?;
        Arc::into_inner(old)
    }

    pub fn get(&self, index: usize) -> Arc<T> {
        let inner = self.inner.read();
        match inner.items[index].as_ref() {
            Some(item) => item.clone(),
            None => panic!(
                "{}",
                Self::empty_slot_access_msg(&inner, index, "ConcurrentSlab::get")
            ),
        }
    }

    /// Improve performance by avoiding `Arc::clone`.
    /// The slab is kept locked while running `f`.
    pub fn map<R>(&self, index: usize, f: impl FnOnce(&T) -> R) -> R {
        let inner = self.inner.read();
        match inner.items[index].as_ref() {
            Some(item) => f(item),
            None => panic!(
                "{}",
                Self::empty_slot_access_msg(&inner, index, "ConcurrentSlab::map")
            ),
        }
    }

    /// Composes the panic message for an access to an empty slot.
    /// Diagnoses the reason for the slot being empty. This method is
    /// on the cold failure path, so the free list lookup is acceptable.
    fn empty_slot_access_msg(inner: &Inner<T>, index: usize, accessor: &str) -> String {
        let reason = if inner.free_list.contains(&index) {
            "the slot was freed"
        } else {
            "the slot is reserved but not yet filled"
        };
        format!("invalid slab index {index} for {accessor}: {reason}")
    }

    pub fn retain(&self, predicate: impl Fn(&usize, &mut Arc<T>) -> bool) {
        let mut inner = self.inner.write();

        let Inner { items, free_list } = &mut *inner;
        for (idx, item) in items.iter_mut().enumerate() {
            if let Some(arc) = item {
                if !predicate(&idx, arc) {
                    free_list.push(idx);
                    item.take();
                }
            }
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write();
        inner.items.clear();
        inner.items.shrink_to(0);

        inner.free_list.clear();
        inner.free_list.shrink_to(0);
    }

    pub fn metrics(&self, slab: String) -> ConcurrentSlabMetrics {
        let inner = self.inner.read();
        ConcurrentSlabMetrics {
            slab,
            length: inner.items.len(),
            capacity: inner.items.capacity(),
            memory_usage: inner.items.capacity() * size_of::<Option<Arc<T>>>(),
            content_memory_usage: (inner.items.len() - inner.free_list.len()) * size_of::<T>(),
            free_slots_length: inner.free_list.len(),
            free_slots_capacity: inner.free_list.capacity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_fill_get() {
        let slab = ConcurrentSlab::<u64>::default();
        let index = slab.reserve();
        assert!(!slab.is_filled(index));

        slab.fill(index, 42);
        assert!(slab.is_filled(index));
        assert_eq!(*slab.get(index), 42);

        // Filling multiple times is allowed, always under the same index.
        slab.fill(index, 43);
        assert_eq!(*slab.get(index), 43);
    }

    #[test]
    #[should_panic(expected = "the slot is reserved but not yet filled")]
    fn get_from_reserved_not_filled_slot_panics() {
        let slab = ConcurrentSlab::<u64>::default();
        let index = slab.reserve();
        let _ = slab.get(index);
    }

    #[test]
    #[should_panic(expected = "the slot was freed")]
    fn get_from_freed_slot_panics() {
        let slab = ConcurrentSlab::<u64>::default();
        let index = slab.insert(42);
        slab.retain(|_, _| false);
        let _ = slab.get(index);
    }

    #[test]
    fn released_reserved_slot_is_reused() {
        let slab = ConcurrentSlab::<u64>::default();
        let index = slab.reserve();
        slab.release_reserved(index);
        // The released slot must be reused by the next reservation or insertion.
        assert_eq!(slab.reserve(), index);
    }

    #[test]
    #[should_panic(expected = "only an empty (reserved and not filled) slot can be released")]
    fn releasing_filled_slot_panics() {
        let slab = ConcurrentSlab::<u64>::default();
        let index = slab.insert(42);
        slab.release_reserved(index);
    }
}
