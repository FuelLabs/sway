library;

use ::hash::*;
use ::storage::storage_api::*;
use ::storage::storage_key::*;

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V> where K: Hash {}

// Helper function to get the storage slot for a tuple (key, field_id)
fn get_storage_slot<K>(tuple: (K, b256)) -> b256 where K: Hash{
    let mut hasher = Hasher::new();
    tuple.0.hash(hasher);
    tuple.1.hash(hasher);
    hasher.sha256()
}

impl<K, V> StorageKey<StorageMap<K, V>> where K: Hash {
    /// Inserts a key-value pair into the map.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired.
    /// * `value` - The value to be stored.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, key: K, value: V) {
        let key = get_storage_slot((key, self.field_id));
        write::<V>(key, 0, value);
    }

    /// Retrieves the `StorageKey` that describes the raw location in storage of the value
    /// stored at `key`, regardless of whether a value is actually stored at that location or not.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    pub fn get(self, key: K) -> StorageKey<V> {
        StorageKey {
            slot: get_storage_slot((key, self.field_id)),
            offset: 0,
            field_id: get_storage_slot((key, self.field_id)),
        }
    }

    /// Clears a value previously stored using a key
    ///
    /// Return a Boolean indicating whether there was a value previously stored at `key`.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let removed = storage.map.remove(key);
    ///     assert(removed);
    ///     assert(storage.map.get(key).is_none());
    /// }
    /// ```
    #[storage(write)]
    pub fn remove(self, key: K) -> bool {
        let key = get_storage_slot((key, self.slot));
        clear::<V>(key)
    }
}
