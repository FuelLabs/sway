use std::collections::HashMap;

/// A HashMap that can hold multiple values and
/// fetch values in a LIFO manner. Rust's MultiMap
/// handles values in a FIFO manner.
pub struct MappedStack<K: std::cmp::Eq + std::hash::Hash, V> {
    container: HashMap<K, Vec<V>>,
}

impl<K: std::cmp::Eq + std::hash::Hash, V> MappedStack<K, V> {
    pub fn new() -> MappedStack<K, V> {
        MappedStack {
            container: HashMap::<K, Vec<V>>::new(),
        }
    }
    pub fn push(&mut self, k: K, v: V) {
        match self.container.get_mut(&k) {
            Some(val_vec) => {
                val_vec.push(v);
            }
            None => {
                self.container.insert(k, vec![v]);
            }
        }
    }
    pub fn get(&self, k: &K) -> Option<&V> {
        self.container.get(k).and_then(|val_vec| val_vec.last())
    }
    pub fn pop(&mut self, k: &K) {
        if let Some(val_vec) = self.container.get_mut(k) {
            val_vec.pop();
            if val_vec.is_empty() {
                self.container.remove(k);
            }
        }
    }
}

impl<K: std::cmp::Eq + std::hash::Hash, V> Default for MappedStack<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
