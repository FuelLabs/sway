use std::{
    fmt,
    sync::{Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Inner<T> {
    pub items: Vec<Option<Arc<RwLock<T>>>>,
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
pub(crate) struct ConcurrentSlabMut<T> {
    pub inner: RwLock<Inner<T>>,
}

impl<T> Clone for ConcurrentSlabMut<T>
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

impl<T> Default for ConcurrentSlabMut<T> {
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

impl<T> ConcurrentSlabMut<T>
where
    T: Clone,
{
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let inner = self.inner.read().unwrap();
        inner.items.len()
    }

    #[allow(dead_code)]
    pub fn values(&self) -> Vec<Arc<RwLock<T>>> {
        let inner = self.inner.read().unwrap();
        inner.items.iter().filter_map(|x| x.clone()).collect()
    }

    pub fn insert(&self, value: T) -> usize {
        self.insert_arc(Arc::new(RwLock::new(value)))
    }

    pub fn insert_arc(&self, value: Arc<RwLock<T>>) -> usize {
        let mut inner = self.inner.write().unwrap();

        if let Some(free) = inner.free_list.pop() {
            assert!(inner.items[free].is_none());
            inner.items[free] = Some(value);
            free
        } else {
            inner.items.push(Some(value));
            inner.items.len() - 1
        }
    }

    pub fn get(&self, index: usize) -> Arc<RwLock<T>> {
        let inner = self.inner.read().unwrap();
        inner.items[index]
            .as_ref()
            .expect("invalid slab index for ConcurrentSlab::get")
            .clone()
    }

    #[allow(dead_code)]
    pub fn retain(&self, predicate: impl Fn(&usize, &mut Arc<RwLock<T>>) -> bool) {
        let mut inner = self.inner.write().unwrap();

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

    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.items.clear();
        inner.items.shrink_to(0);

        inner.free_list.clear();
        inner.free_list.shrink_to(0);
    }
}
