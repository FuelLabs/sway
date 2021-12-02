use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct ConcurrentSlab<T> {
    inner: RwLock<Vec<T>>,
}

impl<T> ConcurrentSlab<T>
where
    T: Clone + PartialEq,
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

    pub fn replace(&self, index: usize, prev_value: &T, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        let actual_prev_value = &inner[index];
        if actual_prev_value != prev_value {
            return Some(actual_prev_value.clone());
        }
        inner[index] = new_value;
        None
    }
}
