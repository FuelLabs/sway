use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use itertools::Itertools;

#[derive(Debug, Clone)]
struct Inner<T> {
    items: Vec<Option<Arc<T>>>,
    free_list: roaring::RoaringBitmap,
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
    inner: RwLock<Inner<T>>,
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
    pub fn values(&self) -> Vec<Arc<T>> {
        let inner = self.inner.read().unwrap();
        inner.items.iter().filter_map(|x| x.clone()).collect()
    }

    pub fn insert(&self, value: T) -> usize {
        self.insert_arc(Arc::new(value))
    }

    pub fn insert_arc(&self, value: Arc<T>) -> usize {
        let mut inner = self.inner.write().unwrap();

        if let Some(free) = inner.free_list.select(0) {
            let free = free as usize;
            assert!(inner.items[free].is_none());
            inner.free_list.remove_smallest(1);
            inner.items[free] = Some(value);
            free
        } else {
            inner.items.push(Some(value));
            inner.items.len() - 1
        }
    }

    pub fn replace(&self, index: usize, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        let item = inner.items.get_mut(index)?;
        let old = item.replace(Arc::new(new_value))?;
        Arc::into_inner(old)
    }

    pub fn get(&self, index: usize) -> Arc<T> {
        let inner = self.inner.read().unwrap();
        inner.items[index]
            .as_ref()
            .expect("invalid slab index for ConcurrentSlab::get")
            .clone()
    }

    pub fn retain(&self, predicate: impl Fn(&usize, &mut Arc<T>) -> bool) {
        let mut inner = self.inner.write().unwrap();

        let Inner { items, free_list } = &mut *inner;
        for (idx, item) in items.iter_mut().enumerate() {
            if let Some(arc) = item {
                if !predicate(&idx, arc) {
                    free_list.insert(u32::try_from(idx).unwrap());
                    item.take();
                }
            }
        }
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.items.clear();
        inner.items.shrink_to(0);

        inner.free_list.clear();
    }
}
