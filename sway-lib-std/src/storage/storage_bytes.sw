library;

use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::storage::storable_slice::*;
use ::storage::{storage_api::*, storage_key::StorageKey};
use ::codec::*;
use ::debug::*;

/// A storage type for storing a collection of tightly packed bytes.
pub struct StorageBytes {}

// Note: `StorageBytes` is a zero-sized storage type that can be nested
//       within other storage types. For example, a `StorageMap<K, StorageBytes>`.
//       That's why we **always use the `self.field_id`** as a storage slot
//       for all of the methods of `StorageBytes`, and **never the `self.slot`**.

#[cfg(experimental_dynamic_storage = false)]
impl StorableSlice<Bytes> for StorageKey<StorageBytes> {
    /// Takes a `Bytes` type and stores the underlying collection of tightly packed bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The bytes which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to read the existing data in the slice length slot)
    /// * Writes: `2` (one for the `Bytes` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     storage.stored_bytes.write_slice(bytes);
    /// }
    /// ```
    #[storage(read, write)]
    fn write_slice(self, bytes: Bytes) {
        write_slice_quads(self.field_id(), bytes.as_raw_slice());
    }

    /// Constructs a `Bytes` type from a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [Option<Bytes>] - The valid `Bytes` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2` (one for the `Bytes` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_none());
    ///     storage.stored_bytes.write_slice(bytes);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice().unwrap();
    ///     assert_eq(bytes, retrieved_bytes);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<Bytes> {
        match read_slice_quads(self.field_id()) {
            Some(slice) => {
                Some(Bytes::from(slice))
            },
            None => None,
        }
    }

    /// Clears stored `Bytes` in storage.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if _all_ of the cleared storage slots were previously set. Otherwise, `false`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to determine the `Bytes` length)
    /// * Clears: `2` (one for the `Bytes` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.write_slice(bytes);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_some());
    ///     let cleared = storage.stored_bytes.clear();
    ///     assert(cleared);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice();
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        clear_slice_quads(self.field_id())
    }

    /// Returns the length of tightly packed bytes in storage, in bytes.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the bytes in storage, in bytes, or `0` if there are no valid bytes in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to read the `Bytes` length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert_eq(storage.stored_bytes.len(), 0);
    ///     storage.stored_bytes.write_slice(bytes);
    ///     assert_eq(storage.stored_bytes.len(), 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read_quads::<u64>(self.field_id(), 0).unwrap_or(0)
    }
}

#[cfg(experimental_dynamic_storage = true)]
impl StorableSlice<Bytes> for StorageKey<StorageBytes> {
    /// Takes a `Bytes` type and stores the underlying collection of tightly packed bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The bytes which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `1` (for storing the `Bytes` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     storage.stored_bytes.write_slice(bytes);
    /// }
    /// ```
    #[storage(write)]
    fn write_slice(self, bytes: Bytes) {
        write_slice_slot(self.field_id(), bytes.as_raw_slice());
    }

    /// Constructs a `Bytes` type from a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [Option<Bytes>] - The valid `Bytes` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1` (for the `Bytes` length)
    /// * Reads: `1` (for the `Bytes` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_none());
    ///     storage.stored_bytes.write_slice(bytes);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice().unwrap();
    ///     assert_eq(bytes, retrieved_bytes);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<Bytes> {
        match read_slice_slot(self.field_id()) {
            Some(slice) => {
                Some(Bytes::from(slice))
            },
            None => None,
        }
    }

    /// Clears stored `Bytes` in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1` (for the `Bytes` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.write_slice(bytes);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_some());
    ///     storage.stored_bytes.clear();
    ///     let retrieved_bytes = storage.stored_bytes.read_slice();
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(write)]
    fn clear(self) {
        clear_slice_slot(self.field_id());
    }

    /// Clears stored `Bytes` in storage and returns whether it existed.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the cleared storage slot was previously set. Otherwise, `false`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preload: `1` (to determine if the slot was previously set)
    /// * Clears: `1` (for the `Bytes` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.write_slice(bytes);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_some());
    ///     let cleared = storage.stored_bytes.clear_existed();
    ///     assert(cleared);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice();
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear_existed(self) -> bool {
        clear_slice_slot_existed(self.field_id())
    }

    /// Returns the length of tightly packed bytes in storage, in bytes.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the bytes in storage, or `0` if there are no valid bytes in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1` (to read the `Bytes` length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert_eq(storage.stored_bytes.len(), 0);
    ///     storage.stored_bytes.write_slice(bytes);
    ///     assert_eq(storage.stored_bytes.len(), 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        __state_preload(self.field_id())
    }
}
