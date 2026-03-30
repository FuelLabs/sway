library;

use ::hash::*;
use ::option::Option;
use ::result::Result;
use ::storage::storage_api::*;
use ::storage::storage_key::*;
use ::codec::*;
use ::debug::*;

/// The storage domain value of the [StorageMap].
///
/// Storage slots of elements contained within a [StorageMap]
/// are calculated based on developers' or users' input (the key).
///
/// To ensure that pre-images used to calculate storage slots can never
/// be the same as a pre-image of a compiler generated key of a storage
/// field, we prefix the pre-images with a single byte that denotes
/// the storage map domain.
///
/// The domain prefix for the [StorageMap] is 1u8.
///
/// For detailed elaboration see: https://github.com/FuelLabs/sway/issues/6317
const STORAGE_MAP_DOMAIN: u8 = 1;

/// Errors pertaining to the `StorageMap` struct.
pub enum StorageMapError<V> {
    /// Indicates that a value already exists for the key.
    OccupiedError: V,
}

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V> {}

// Methods for `StorageMap` that are same regardless of the value
// of the `experimental_dynamic_storage` feature flag.
impl<K, V> StorageKey<StorageMap<K, V>>
where
    K: Hash,
{
    /// Retrieves the `StorageKey` that describes the location in storage of the value
    /// stored at `key`, regardless of whether a value is actually stored at that location or not.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    ///
    /// # Returns
    ///
    /// * [StorageKey<V>] - Describes the location in storage of the value stored at `key`.
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
    ///     assert_eq(value, retrieved_value);
    /// }
    /// ```
    pub fn get(self, key: K) -> StorageKey<V>
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);
        StorageKey::<V>::new(key, 0, key)
    }

    // Note: `StorageMap` is a zero-sized storage type that can be nested
    //       within other storage types. For example, a `StorageMap<K, StorageMap>`.
    //       That's why we are **using the `self.field_id`** for getting the storage slot
    //       for all of the methods of `StorageMap`, and **not the `self.slot`**.
    fn get_slot_key(self, key: K) -> b256 {
        sha256((STORAGE_MAP_DOMAIN, key, self.field_id()))
    }
}

#[cfg(experimental_dynamic_storage = false)]
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
    /// * Reads: `0` if the `value` occupies full slots, `1` otherwise (to read the existing data that will be partially overwritten)
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
    ///     assert_eq(value, retrieved_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, key: K, value: V)
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);
        write_quads::<V>(key, 0, value);
    }

    /// Clears a value previously stored at `key`.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if there was a value previously stored at `key`.
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
        let key = self.get_slot_key(key);
        clear_quads::<V>(key, 0)
    }

    /// Inserts a key-value pair into the map if a value does not already exist for the `key`.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    /// * `value`: [V] - The value to be stored.
    ///
    /// # Returns
    ///
    /// * [Result<V, StorageMapError<V>>] - `Result::Ok(value)` if the `value` was inserted, or `Result::Err(StorageMapError::OccupiedError(pre_existing_value))` if a value already existed for the `key`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to check if a value already exists for the `key`)
    /// * Writes: `1` if the `value` is inserted, otherwise `0`
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
    ///     assert_eq(value, retrieved_value); // New value was not inserted, as a value already existed.
    ///
    ///     let key2 = 10_u64;
    ///     let returned_value = storage.map.try_insert(key2, new_value);
    ///     assert_eq(returned_value, Result::Ok(new_value)); // New value is returned.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn try_insert(self, key: K, value: V) -> Result<V, StorageMapError<V>>
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);

        let val = read_quads::<V>(key, 0);

        match val {
            Option::Some(v) => {
                Result::Err(StorageMapError::OccupiedError(v))
            },
            Option::None => {
                write_quads::<V>(key, 0, value);
                Result::Ok(value)
            }
        }
    }
}

#[cfg(experimental_dynamic_storage = true)]
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
    ///     assert_eq(value, retrieved_value);
    /// }
    /// ```
    #[storage(write)]
    pub fn insert(self, key: K, value: V)
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);
        write_slot::<V>(key, value);
    }

    /// Clears a value previously stored at `key`.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if there was a value previously stored at `key`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1` (to check whether the slot was previously set)
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
    ///     let removed = storage.map.remove_existed(key);
    ///     assert(removed);
    ///     assert(storage.map.get(key).is_none());
    /// }
    /// ```
    #[storage(write)]
    pub fn remove_existed(self, key: K) -> bool
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);
        clear_slots_existed(key, 1)
    }

    /// Clears a value previously stored at `key`.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
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
    ///     storage.map.remove(key);
    ///     assert(storage.map.get(key).is_none());
    /// }
    /// ```
    #[storage(write)]
    pub fn remove(self, key: K)
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);
        clear_slots(key, 1)
    }

    /// Inserts a key-value pair into the map if a value does not already exist for the `key`.
    ///
    /// # Arguments
    ///
    /// * `key`: [K] - The key to which the value is paired.
    /// * `value`: [V] - The value to be stored.
    ///
    /// # Returns
    ///
    /// * [Result<V, StorageMapError<V>>] - `Result::Ok(value)` if the `value` was inserted, or `Result::Err(StorageMapError::OccupiedError(pre_existing_value))` if a value already existed for the `key`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to check if a value already exists for the `key`)
    /// * Writes: `1` if the `value` is inserted, otherwise `0`
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
    ///     assert_eq(value, retrieved_value); // New value was not inserted, as a value already existed.
    ///
    ///     let key2 = 10_u64;
    ///     let returned_value = storage.map.try_insert(key2, new_value);
    ///     assert_eq(returned_value, Result::Ok(new_value)); // New value is returned.
    /// }
    /// ```
    #[storage(read, write)]
    pub fn try_insert(self, key: K, value: V) -> Result<V, StorageMapError<V>>
    where
        K: Hash,
    {
        let key = self.get_slot_key(key);

        let val = read_slot::<V>(key, 0);

        match val {
            Option::Some(v) => {
                Result::Err(StorageMapError::OccupiedError(v))
            },
            Option::None => {
                write_slot::<V>(key, value);
                Result::Ok(value)
            }
        }
    }
}
