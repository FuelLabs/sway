library;

use ::hash::*;
use ::option::Option::{self, *};
use ::result::Result;
use ::storage::storage_api::*;
use ::storage::storage_key::*;

/// Errors pertaining to the `StorageMap` struct.
pub enum StorageMapError<V> {
    /// Indicates that a value already exists for the key.
    OccupiedError: V,
}

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V>
where
    K: Hash,
{
    id: StorageKey
}

impl<K, V> Storage for StorageMap<K, V>
where
    K: Hash,
{
    fn new(id: StorageKey) -> Self {
        StorageMap {
            id
        }
    }
}

impl<K, V> StorageMap<K, V>
where
    K: Hash,
    V: Storage,
{
    fn index_key(self, index: K) -> StorageKey {
        let slot = sha256((index, self.id.slot));
        StorageKey::new(slot, 0)
    }
    /// Inserts a key-value pair into the map.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    /// * `value`: [V] - The value to be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `1`
    ///
    /// # Examples
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
    pub fn insert(self, key: K, value: V)
    {
        self.index_key(key).write::<V>(value);
    }

    /// Retrieves the `StorageKey` that describes the raw location in storage of the value
    /// stored at `key`, regardless of whether a value is actually stored at that location or not.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    ///
    /// # Returns
    ///
    /// * [StorageKey<V>] - Describes the raw location in storage of the value stored at `key`.
    ///
    /// # Examples
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
    pub fn get(self, key: K) -> V
    {
        V::new(self.index_key(key))
    }

    /// Inserts a key-value pair into the map if a value does not already exist for the key.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    /// * `value`: [V] - The value to be stored.
    ///
    /// # Returns
    ///
    /// * [Result<V, StorageMapError<V>>] - `Result::Ok(value)` if the value was inserted, or `Result::Err(StorageMapError::OccupiedError(pre_existing_value))` if a value already existed for the key.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::storage::storage_map::StorageMapError;
    ///
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///
    ///     let new_value = false;
    ///     let result = storage.map.try_insert(key, new_value);
    ///     assert(result == Result::Err(StorageMapError::OccupiedError(value))); // The old value is returned.
    ///
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value); // New value was not inserted, as a value already existed.
    ///
    ///     let key2 = 10_u64;
    ///     let returned_value = storage.map.try_insert(key2, new_value);
    ///     assert(returned_value == Result::Ok(new_value)); // New value is returned.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn try_insert(self, key: K, value: V) -> Result<V, StorageMapError<V>>
{
        let key = self.index_key(key);

        match key.read::<V>() {
            Some(v) => {
                Result::Err(StorageMapError::OccupiedError(v))
            },
            None => {
                key.write::<V>(value);
                Result::Ok(value)
            }
        }
    }
}
