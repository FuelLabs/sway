library;

use ::hash::*;
use ::option::Option;
use ::storage::storage_api::*;
use ::storage::storage_key::*;

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V>
where
    K: Hash,
{}

impl<K, V> StorageKey<StorageMap<K, V>>
where
    K: Hash,
{
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
    where
        K: Hash,
{
        let key = sha256((key, self.field_id));
        write::<V>(key, 0, value);
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
    pub fn get(self, key: K) -> StorageKey<V>
    where
        K: Hash,
{
        StorageKey::<V>::new(
            sha256((key, self.field_id)),
            0,
            sha256((key, self.field_id)),
        )
    }

    /// Clears a value previously stored using a key
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether there was a value previously stored at `key`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1`
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
    ///     let removed = storage.map.remove(key);
    ///     assert(removed);
    ///     assert(storage.map.get(key).is_none());
    /// }
    /// ```
    #[storage(write)]
    pub fn remove(self, key: K) -> bool
    where
        K: Hash,
{
        let key = sha256((key, self.slot));
        clear::<V>(key, 0)
    }

    /// Inserts a key-value pair into the map, if a value does not already exist for the key.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    /// * `value`: [V] - The value to be stored.
    ///
    /// # Returns
    ///
    /// * [Option<V>] - The value previously stored at `key`, if any.
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
    ///     
    ///     let new_value = false;
    ///     let old_value = storage.map.try_insert(key, new_value);
    ///     assert(old_value == Option::Some(value)); // The old value is returned.
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value); // New value was not inserted, as a value already existed.
    ///
    ///     let key2 = 10_u64;
    ///     let old_value = storage.map.try_insert(key2, new_value);
    ///     assert(old_value == Option::None); // No old value is returned.
    ///     let retrieved_value = storage.map.get(key2).read();
    ///     assert(new_value == retrieved_value); // New value was inserted, as no value existed prior.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn try_insert(self, key: K, value: V) -> Option<V>
    where K: Hash, 
    {
        let key = sha256((key, self.field_id));
        
        let val = read::<V>(key, 0);

        match val {
            Option::Some(v) => {Option::Some(v)},
            Option::None => {
                write::<V>(key, 0, value);
                Option::None
            }
        }
    }
}
