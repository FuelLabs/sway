use std::{
    fmt,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub(crate) struct ConcurrentSlab<T> {
    inner: RwLock<Vec<Arc<T>>>,
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
    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.len()
    }

    pub fn insert(&self, value: T) -> usize {
        let mut inner = self.inner.write().unwrap();
        let ret = inner.len();
        inner.push(Arc::new(value));
        ret
    }

    pub fn insert_arc(&self, value: Arc<T>) -> usize {
        let mut inner = self.inner.write().unwrap();
        let ret = inner.len();
        inner.push(value);
        ret
    }

    pub fn replace(&self, index: usize, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        inner[index] = Arc::new(new_value);
        None
    }

    pub fn get(&self, index: usize) -> Arc<T> {
        let inner = self.inner.read().unwrap();
        inner[index].clone()
    }

    pub fn retain(&self, predicate: impl Fn(&Arc<T>) -> bool) {
        let mut inner = self.inner.write().unwrap();
        inner.retain(predicate);
    }
}
