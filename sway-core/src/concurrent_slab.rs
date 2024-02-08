use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<T> {
    inner: RwLock<HashMap<usize, Arc<T>>>,
    last_id: Arc<RwLock<usize>>,
}

impl<T> Clone for ConcurrentSlab<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let inner = self.inner.read().unwrap();
        Self {
            inner: RwLock::new(inner.clone()),
            last_id: self.last_id.clone(),
        }
    }
}

impl<T> Default for ConcurrentSlab<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            last_id: Default::default(),
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
        inner.values().cloned().collect_vec()
    }

    pub fn insert(&self, value: T) -> usize {
        let mut inner = self.inner.write().unwrap();
        let mut last_id = self.last_id.write().unwrap();
        *last_id += 1;
        inner.insert(*last_id, Arc::new(value));
        *last_id
    }

    pub fn insert_arc(&self, value: Arc<T>) -> usize {
        let mut inner = self.inner.write().unwrap();
        let mut last_id = self.last_id.write().unwrap();
        *last_id += 1;
        inner.insert(*last_id, value);
        *last_id
    }

    pub fn replace(&self, index: usize, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        inner.insert(index, Arc::new(new_value));
        None
    }

    pub fn get(&self, index: usize) -> Arc<T> {
        let inner = self.inner.read().unwrap();
        inner[&index].clone()
    }

    pub fn retain(&self, predicate: impl Fn(&usize, &mut Arc<T>) -> bool) {
        let mut inner = self.inner.write().unwrap();
        inner.retain(predicate);
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.clear();
        inner.shrink_to(0);
    }
}
