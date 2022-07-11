use crate::type_engine::TypeId;
use std::sync::RwLock;

#[derive(Debug, Default)]
pub struct ConcurrentSlab<T> {
    inner: RwLock<Vec<T>>,
}

impl<T> ConcurrentSlab<T>
where
    T: Clone + PartialEq,
{
    pub fn insert(&self, value: T) -> TypeId {
        let mut inner = self.inner.write().unwrap();
        let ret = inner.len();
        inner.push(value);
        ret.into()
    }

    pub fn get(&self, index: TypeId) -> T {
        let inner = self.inner.read().unwrap();
        inner[*index].clone()
    }

    pub fn replace(&self, index: TypeId, prev_value: &T, new_value: T) -> Option<T> {
        let mut inner = self.inner.write().unwrap();
        let actual_prev_value = &inner[*index];
        if actual_prev_value != prev_value {
            return Some(actual_prev_value.clone());
        }
        inner[*index] = new_value;
        None
    }

    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        *inner = Vec::new();
    }
}
